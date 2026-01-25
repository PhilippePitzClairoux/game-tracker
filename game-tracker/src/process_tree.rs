use std::collections::{BTreeMap, HashMap};
use std::collections::btree_map::{Iter, IterMut};
use std::hash::{Hash};
use chrono::{Utc};
use sysinfo::{Pid, Process};

/// ProcessInfo represents a running process. It is based off sysinfo::Process.
/// The key differences are the hash functions (that way we can insert them in a BTree
/// in order to optimize searches/inserts).
#[derive(Debug, Clone)]
pub struct ProcessInfo {
    children: Option<ProcessTree>,
    name: String,
    cmd: Vec<String>,
    pid: Pid,
    run_time: u64,
    start_time: u64,
}

impl Eq for ProcessInfo {
    fn assert_receiver_is_total_eq(&self) {
        assert_eq!(self, self)
    }
}

impl PartialEq<Self> for ProcessInfo {

    fn eq(&self, other: &Self) -> bool {
        self.pid == other.pid &&
            self.cmd == other.cmd &&
            self.name == other.name
    }
}

impl Hash for ProcessInfo {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.pid.hash(state);
        self.cmd.hash(state);
        self.name.hash(state);
    }
}

impl ProcessInfo {

    pub fn from(proc: &Process) -> ProcessInfo {
        ProcessInfo {
            children: None,
            name: proc.name().to_string_lossy().to_string(),
            cmd: proc.cmd().iter().map(|s| s.to_string_lossy().to_string()).collect(),
            run_time: proc.run_time(),
            pid: proc.pid().clone(),
            start_time: proc.start_time()
        }
    }

    fn insert_child(&mut self, pid: Pid, child: ProcessInfo) {
        match self.children {
            Some(ref mut proc_tree) => {
                proc_tree.insert(pid, Box::new(child));
            }
            None => {
                self.children = Some(ProcessTree::new());
                self.children.as_mut().unwrap().insert(pid, Box::new(child));
            }
        }
    }

    pub fn cmd(&self) -> String {
        self.cmd.join(" ")
    }

    pub fn run_time(&self) -> u64 {
        self.run_time
    }

    pub fn pid(&self) -> Pid { self.pid }

    pub fn children(&self) -> &Option<ProcessTree> { &self.children }

    pub fn cmd_contains(&self, s: &str) -> bool {
        self.cmd().contains(s)
    }

    pub fn name(&self) -> &str { &self.name }

    pub fn start_time(&self) -> u64 { self.start_time }

    /// This function searches a string `s` inside a ProcessInfo (cmd and name)
    /// Goes through the list of children (if they are present)
    pub fn find(&self, s: &str) -> Option<&ProcessInfo> {
        if self.cmd_contains(s) || self.name.contains(s) {
            return Some(self)
        }

        match self.children {
            Some(ref children) => {
                for (_, n) in children.iter() {
                    match n.find(s) {
                        Some(found) => return Some(found),
                        None => ()
                    };
                }
            }
            None => ()
        }

        None
    }

    #[allow(dead_code)]
    pub fn to_string(&self, level: usize) -> String {
        let mut output = String::from(
            format!("{}|__<{}> {}\n", " ".repeat(level), self.pid, self.cmd())
        );

        if let Some(children) = self.children.as_ref() {
            for (_, v) in children.iter() {
                output += v.to_string(level+1).as_str();
            }
        }

        output
    }

}

/// This class represents a process tree. It's a BTree of ProcessInfo.
/// This structure is recursive and is basically a wrapper around BTreeMap
/// to facilitate searching.
#[derive(Debug, Clone)]
pub struct ProcessTree {
    inner: BTreeMap<Pid, Box<ProcessInfo>>
}

impl ProcessTree {

    pub fn new() -> ProcessTree {
        ProcessTree{
            inner: BTreeMap::new()
        }
    }

    fn insert_process(&mut self, proc: &Process, parent: Pid) -> bool {
        if self.contains_key(&parent) {
            self.inner.get_mut(&parent).unwrap()
                .insert_child(proc.pid(), ProcessInfo::from(proc));
            return true;
        }

        for node in self.inner.values_mut() {
            if let Some(children) = node.children.as_mut() {
                if children.insert_process(proc, parent) {
                    return true;
                }
            }
        }

        false
    }


    pub fn from(processes: &HashMap<Pid, Process>) -> ProcessTree {
        let mut tree: ProcessTree = ProcessTree::new();

        BTreeMap::from_iter(processes.iter()).iter()
            .for_each(|(pid, process)| {
                if let Some(parent) = process.parent() && parent.as_u32() != 1 {
                    tree.insert_process(process, process.parent().unwrap());
                } else {
                    tree.inner.insert(
                        Pid::from_u32(pid.as_u32()),
                        Box::new(ProcessInfo::from(process))
                    );
                }
            });

        tree
    }

    pub fn iter(&self) -> Iter<'_, Pid, Box<ProcessInfo>> {
        self.inner.iter()
    }

    pub fn insert(&mut self, pid: Pid, process: Box<ProcessInfo>) -> Option<Box<ProcessInfo>> {
        self.inner.insert(pid, process)
    }

    pub fn contains_key(&self, pid: &Pid) -> bool {
        self.inner.contains_key(pid)
    }

}
