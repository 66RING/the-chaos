#![allow(dead_code)]

const MAX_STAGE_NUM: usize = 130;
/// stage flag: whether it is a valid end stage
const END_STAGE: usize = MAX_STAGE_NUM - 1;
const INVALID_STAGE: usize = 0;

#[derive(Clone, Copy, Default)]
struct FsmAction {
    next: usize,
    // 待匹配字符串移位量
    offset: i32,
}

/// index of next stage
#[derive(Clone, Copy)]
struct NextStageIndex([FsmAction; MAX_STAGE_NUM]);

struct Regex {
    stages: Vec<NextStageIndex>,
}

/// 0 as end stage
impl NextStageIndex {
    pub fn new() -> Self {
        Self ([Default::default(); MAX_STAGE_NUM])
    }
}

impl Regex {

    pub fn compile(tokens: &str) -> Regex {
        /// init with 0 stage
        let mut fsm = Self { stages: Vec::new() };
        fsm.stages.push(NextStageIndex::new());

        for c in tokens.chars() {
            let mut new_stage = NextStageIndex::new();
            let next_index = fsm.stages.len() + 1;
            let current_stage = fsm.stages.len();
            match c {
                '$' => {
                    // set stage[END_STAGE] as the end flag
                    new_stage.0[END_STAGE] = FsmAction {
                        next: next_index,
                        offset: 1,
                    };
                    fsm.stages.push(new_stage)
                },
                '.' => {
                    // printable char
                    for i in 32..127 {
                        new_stage.0[i].next = next_index;
                        new_stage.0[i].offset = 1;
                    }
                    fsm.stages.push(new_stage)
                },
                '*' => {
                    // consume any more of current stage
                    for t in fsm.stages.last_mut().unwrap().0.iter_mut() {
                        if t.next == current_stage {
                            // find last stage(next = n) and change it
                            t.next = current_stage - 1;
                            t.offset = 1;
                        } else if t.next == 0 {
                            // **如果遇到其他char则可以跳到下一个stage**
                            t.next = current_stage;
                            // 但str轮空
                            t.offset = 0;
                        } else {
                            unreachable!()
                        }
                    }
                },
                _ => {
                    new_stage.0[c as usize].next = next_index;
                    new_stage.0[c as usize].offset = 1;
                    fsm.stages.push(new_stage)
                }
            }
        }

        fsm
    }

    pub fn match_str(&self, input: &str) -> bool {
        let mut stage = 1;
        // pointer of str
        let mut head = 0;
        let chars = input.chars().collect::<Vec<_>>();
        let n = input.len();

        while stage > 0 && stage < self.stages.len() && head < n {
            let action = self.stages[stage].0[chars[head] as usize];
            stage = action.next;
            head = (head as i32 + action.offset) as usize;
        }

        if stage == INVALID_STAGE {
            return false
        }

        // check if end stage flag is set. i.e. if this stage is a valid end stage
        if stage < self.stages.len() {
            stage = self.stages[stage].0[END_STAGE].next;
        }

        // if all match or reach end stage
        return stage >= self.stages.len() || self.stages[stage].0[END_STAGE].next != 0;
    }

    ///       stage1 stage2
    /// next1
    /// next2
    pub fn dump_stage(&self) {
        let table = Vec::<Vec<usize>>::with_capacity(self.stages.len());
        let mut row = 0;
        while row < MAX_STAGE_NUM {
            print!("{:03} ", row);
            for s in &self.stages {
                print!("({}, {}) ", s.0[row].next, s.0[row].offset);
            }
            println!();
            row += 1;
        }
    }
}

fn test_regex(regex_str: &str, test_cases: &[(&str, bool)]) {
    let reg = Regex::compile(regex_str);

    for (case, expected) in test_cases {
        assert_eq!(reg.match_str(case), *expected);
    }
}

fn main() {
    let reg = Regex::compile("bc.*");
    reg.dump_stage();
    println!("{}", reg.match_str("bc"));
    println!("{}", reg.match_str("bcadsf"));
    println!("{}", reg.match_str("abccc"));
}

mod test {
    use crate::test_regex;


    #[test]
    fn basic() {

        let tests = vec![
            (
                "abc*d$",
                vec![("abcd", true), ("abc", false), ("abcccccd", true)],
                true, // enable
            ),
            (
                ".*",
                vec![
                    ("abcd", true), ("abc", true), ("abcccccd", true),
                    ("adsfcd", true), ("", true), ("vcbpore", true),
                ],
                true, // enable
            ),
            (
                ".*bc", 
                vec![
                    // ("bc", true),
                    ("abc", true),
                    ("aabc", true),
                ],
                true,
            ),
        ];

        for (regex_str, case, enable) in tests {
            if enable != true {
                continue;
            }
            test_regex(regex_str, &case);
        }
    }
}
