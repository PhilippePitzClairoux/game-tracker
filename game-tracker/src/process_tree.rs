use std::collections::{BTreeMap, HashMap};
use std::collections::btree_map::{Iter, IterMut};
use std::hash::{Hash};
use chrono::{Utc};
use sysinfo::{Pid, Process};

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

    pub fn new() -> ProcessInfo {
        ProcessInfo {
            children: None,
            name: String::new(),
            cmd: Vec::new(),
            run_time: 0,
            pid: Pid::from_u32(0),
            start_time: Utc::now().timestamp() as u64,
        }
    }

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

    pub fn is_child_present(&self, p: &Pid) -> bool {
        self.children.as_ref()
            .is_some_and(|child| child.contains_key(p))
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

    pub fn find(&self, s: &str) -> Option<&ProcessInfo> {
        if self.cmd_contains(s) || self.name.contains(s) {
            return Some(self)
        }

        if self.children.is_some() {
            for (_, n) in self.children.as_ref().unwrap().iter() {
                match n.find(s) {
                    Some(found) => return Some(found),
                    None => ()
                }
            }
        }

        None
    }

    #[allow(dead_code)]
    pub fn to_string(&self, level: usize) -> String {
        let mut output = String::from(
            format!("{}|__<{}> {}\n", " ".repeat(level), self.pid, self.cmd())
        );

        if self.children.is_some() {
            for (_, v) in self.children.as_ref().unwrap().iter() {
                output += v.to_string(level+1).as_str();
            }
        }

        output
    }

}

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
            if node.children.is_some() {
                if node.children.as_mut().unwrap().insert_process(proc, parent) {
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
                if process.parent().is_some() && process.parent().unwrap().as_u32() != 1 {
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

    pub fn iter_mut(&mut self) -> IterMut<'_, Pid, Box<ProcessInfo>> {
        self.inner.iter_mut()
    }

    pub fn insert(&mut self, pid: Pid, process: Box<ProcessInfo>) -> Option<Box<ProcessInfo>> {
        self.inner.insert(pid, process)
    }

    pub fn contains_key(&self, pid: &Pid) -> bool {
        self.inner.contains_key(pid)
    }

}
