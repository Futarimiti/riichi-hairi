use super::{Hai, Haiyama, Mentsu, PlayerNumber, Tehai};
use serde_json::json;
use std::collections::BTreeSet;

/// The game manager.
/// Include everything that a complete mahjong game need.
#[derive(Clone, Debug)]
pub struct GameManager {
    haiyama: Haiyama,
    tehai: Option<Tehai>,
    sutehai_type: BTreeSet<Hai>,
    pub state: State,
    history: Vec<(Operation, State)>,
}

/// Type of kan.
///
/// # Japanese
/// * Daiminkan: 大明槓
/// * Kakan: 加槓
/// * Ankan: 暗槓
/// * kantsu: 槓子
/// * rinshanhai: 嶺上牌
#[derive(Clone, Debug)]
pub enum Kan {
    Daiminkan {
        kantsu: Mentsu,
        rinshanhai: Option<Hai>,
    },
    Kakan {
        kantsu: Mentsu,
        rinshanhai: Option<Hai>,
    },
    Ankan {
        kantsu: Mentsu,
        rinshanhai: Option<Hai>,
    },
    Unknown {
        kantsu: Mentsu,
        rinshanhai: Option<Hai>,
    },
}

/// Type of naku.
///
/// # Japanese
/// * Naku: 鳴く
/// * Chii: チー
/// * Pon: ポン
/// * Kan: カン
/// * nakihai: 鳴き牌
#[derive(Clone, Debug)]
pub enum Naku {
    Chii { juntsu: Mentsu, nakihai: Hai },
    Pon(Mentsu),
    Kan(Kan),
}

/// Operation on haiyama.
#[derive(Clone, Debug)]
pub enum HaiyamaOperation {
    Add(Vec<Hai>),
    Discard(Vec<Hai>),
}

/// Operation on tehai.
#[derive(Clone, Debug)]
pub enum TehaiOperation {
    Initialize(Tehai),
    Add { hai: Hai, bound_check: bool },
    Discard(Hai),
    Naku { kind: Naku, bound_check: bool },
}

/// Valid operation for game manager.
#[derive(Clone, Debug)]
pub enum Operation {
    Haiyama {
        kind: HaiyamaOperation,
        bound_check: bool,
    },
    Tehai(TehaiOperation),
}

/// Game state.
#[derive(Copy, Clone, Debug)]
pub enum State {
    WaitToInit,
    FullHai,
    LackOneHai,
    WaitForRinshanhai,
}

impl GameManager {
    /// Create a instance of GameManager.
    pub fn new(player_number: PlayerNumber) -> Self {
        Self {
            haiyama: Haiyama::new(player_number),
            tehai: None,
            sutehai_type: BTreeSet::new(),
            state: State::WaitToInit,
            history: vec![],
        }
    }

    /// Return a reference of haiyama
    pub fn haiyama(&self) -> &Haiyama {
        &self.haiyama
    }

    /// Return a reference of the set within sutehai.
    pub fn sutehai_type(&self) -> &BTreeSet<Hai> {
        &self.sutehai_type
    }

    pub fn history(&self) -> &Vec<(Operation, State)> {
        &self.history
    }

    /// Main function to control the game.
    pub fn operate(&mut self, mut op: Operation) -> Result<(), String> {
        let last_state = self.state;
        match last_state {
            State::WaitToInit => self.operate_wait_to_init(&op)?,
            State::FullHai => self.operate_full_hai(&mut op)?,
            State::LackOneHai => self.operate_lack_one_hai(&mut op)?,
            State::WaitForRinshanhai => self.operate_wait_for_rinshanhai(&op)?,
        }
        self.history.push((op, last_state));
        Ok(())
    }

    /// Print self to json.
    pub fn to_json(&self) -> serde_json::Value {
        let mut sutehai_type_string_vec = vec![];
        for hai in self.sutehai_type.iter() {
            sutehai_type_string_vec.push(hai.to_string());
        }

        let tehai_json = match &self.tehai {
            Some(tehai) => tehai.to_json(),
            None => json!("Not initialized."),
        };

        json!({
            "haiyama": self.haiyama.to_json(),
            "sutehai_type": json!(sutehai_type_string_vec),
            "tehai": tehai_json,
        })
    }

    fn operate_wait_to_init(&mut self, op: &Operation) -> Result<(), String> {
        fn operate_tehai_init(self_: &mut GameManager, tehai: &Tehai) -> Result<(), String> {
            if tehai.fuuro.len() != 0 {
                return Err("Cannot initialized with fuuro.".to_string());
            }
            match tehai.juntehai.len() {
                13 => self_.state = State::LackOneHai,
                14 => self_.state = State::FullHai,
                num @ _ => {
                    return Err(format!(
                        "Cannot initialize tehai with {} juntehai, only 13 and 14 are supported.",
                        num
                    ))
                }
            }
            if let Err(error) = self_.haiyama.discard_with_vec(&tehai.juntehai, true) {
                self_.state = State::WaitToInit;
                return Err(error);
            }

            Ok(())
        }

        match &op {
            Operation::Tehai(TehaiOperation::Initialize(tehai)) => {
                operate_tehai_init(self, tehai)?;
            }
            Operation::Haiyama {
                kind: HaiyamaOperation::Add(hai_vec),
                bound_check,
            } => {
                if let Err(error) = self.haiyama.add_with_vec(hai_vec, *bound_check) {
                    if *bound_check {
                        return Err(error);
                    }
                }
            }
            Operation::Haiyama {
                kind: HaiyamaOperation::Discard(hai_vec),
                bound_check,
            } => {
                if let Err(error) = self.haiyama.discard_with_vec(hai_vec, *bound_check) {
                    if *bound_check {
                        return Err(error);
                    }
                }
            }
            _ => {
                return Err(format!(
                    "Unsupported opretion '{:?}' at state '{:?}'.",
                    op, self.state
                ))
            }
        }

        Ok(())
    }

    fn operate_full_hai(&mut self, op: &mut Operation) -> Result<(), String> {
        match &*op {
            Operation::Tehai(TehaiOperation::Discard(hai)) => {
                self.tehai.as_mut().unwrap().discard(hai)?;
                self.state = State::LackOneHai;
            }
            Operation::Tehai(TehaiOperation::Naku {
                kind: Naku::Kan(Kan::Unknown { kantsu, rinshanhai }),
                bound_check,
            }) => {
                let haiyama_backup = self.haiyama.clone();
                let state_backup = self.state;
                let tehai_backup = self.tehai.clone();
                if let Some(rinshanhai) = rinshanhai {
                    if let Err(error) = self.haiyama.discard(rinshanhai) {
                        if *bound_check {
                            return Err(error);
                        }
                    }
                    self.state = State::FullHai;
                } else {
                    self.state = State::WaitForRinshanhai;
                }

                match self.tehai.as_mut().unwrap().kan(kantsu, rinshanhai) {
                    Ok(kan) => {
                        if let Kan::Ankan { .. } | Kan::Kakan { .. } = &kan {
                            *op = Operation::Tehai(TehaiOperation::Naku {
                                kind: Naku::Kan(kan),
                                bound_check: *bound_check,
                            })
                        } else {
                            self.haiyama = haiyama_backup;
                            self.state = state_backup;
                            self.tehai = tehai_backup;
                            return Err(
                                "Logic error: Tehai currently is not able to kan.".to_string()
                            );
                        }
                    }
                    Err(error) => {
                        self.haiyama = haiyama_backup;
                        self.state = state_backup;
                        return Err(error);
                    }
                }
            }
            Operation::Haiyama {
                kind: HaiyamaOperation::Add(hai_vec),
                bound_check,
            } => {
                if let Err(error) = self.haiyama.add_with_vec(hai_vec, *bound_check) {
                    if *bound_check {
                        return Err(error);
                    }
                }
            }
            Operation::Haiyama {
                kind: HaiyamaOperation::Discard(hai_vec),
                bound_check,
            } => {
                if let Err(error) = self.haiyama.discard_with_vec(hai_vec, *bound_check) {
                    if *bound_check {
                        return Err(error);
                    }
                }
            }
            _ => {
                return Err(format!(
                    "Unsupported opretion '{:?}' at state '{:?}'.",
                    op, self.state
                ))
            }
        }

        Ok(())
    }

    fn operate_lack_one_hai(&mut self, op: &mut Operation) -> Result<(), String> {
        match &*op {
            Operation::Tehai(TehaiOperation::Add { hai, bound_check }) => {
                if let Err(error) = self.haiyama.discard(hai) {
                    if *bound_check {
                        return Err(error);
                    }
                }
                self.tehai.as_mut().unwrap().juntehai.push(*hai);
                self.tehai.as_mut().unwrap().juntehai.sort();
                self.state = State::FullHai;
            }
            Operation::Tehai(TehaiOperation::Naku {
                kind: Naku::Chii { juntsu, nakihai },
                bound_check,
            }) => {
                let haiyama_backup = self.haiyama.clone();
                if let Err(error) = self.haiyama.discard(nakihai) {
                    if *bound_check {
                        return Err(error);
                    }
                }
                if let Err(error) = self.tehai.as_mut().unwrap().chii(juntsu, nakihai) {
                    self.haiyama = haiyama_backup;
                    return Err(error);
                }
                self.state = State::FullHai;
            }
            Operation::Tehai(TehaiOperation::Naku {
                kind: Naku::Pon(koutsu @ Mentsu::Koutsu(hai)),
                bound_check,
            }) => {
                let haiyama_backup = self.haiyama.clone();
                if let Err(error) = self.haiyama.discard(hai) {
                    if *bound_check {
                        return Err(error);
                    }
                }
                if let Err(error) = self.tehai.as_mut().unwrap().pon(koutsu) {
                    self.haiyama = haiyama_backup;
                    return Err(error);
                }
                self.state = State::FullHai;
            }
            Operation::Tehai(TehaiOperation::Naku {
                kind:
                    Naku::Kan(Kan::Unknown {
                        kantsu: kantsu @ Mentsu::Kantsu(hai),
                        rinshanhai,
                    }),
                bound_check,
            }) => {
                let haiyama_backup = self.haiyama.clone();
                let state_backup = self.state;
                let tehai_backup = self.tehai.clone();
                if let Err(error) = self.haiyama.discard(hai) {
                    if *bound_check {
                        return Err(error);
                    }
                }
                if let Some(rinshanhai) = rinshanhai {
                    if let Err(error) = self.haiyama.discard(rinshanhai) {
                        if *bound_check {
                            self.haiyama = haiyama_backup;
                            return Err(error);
                        }
                    }
                    self.state = State::FullHai;
                } else {
                    self.state = State::WaitForRinshanhai;
                }

                match self.tehai.as_mut().unwrap().kan(kantsu, rinshanhai) {
                    Ok(kan) => {
                        if let Kan::Daiminkan { .. } = &kan {
                            *op = Operation::Tehai(TehaiOperation::Naku {
                                kind: Naku::Kan(kan),
                                bound_check: *bound_check,
                            })
                        } else {
                            self.haiyama = haiyama_backup;
                            self.state = state_backup;
                            self.tehai = tehai_backup;
                            return Err(
                                "Logic error: Tehai currently is not able to kan.".to_string()
                            );
                        }
                    }
                    Err(error) => {
                        self.haiyama = haiyama_backup;
                        self.state = state_backup;
                        return Err(error);
                    }
                }
            }
            Operation::Haiyama {
                kind: HaiyamaOperation::Add(hai_vec),
                bound_check,
            } => {
                if let Err(error) = self.haiyama.add_with_vec(hai_vec, *bound_check) {
                    if *bound_check {
                        return Err(error);
                    }
                }
            }
            Operation::Haiyama {
                kind: HaiyamaOperation::Discard(hai_vec),
                bound_check,
            } => {
                if let Err(error) = self.haiyama.discard_with_vec(hai_vec, *bound_check) {
                    if *bound_check {
                        return Err(error);
                    }
                }
            }
            _ => {
                return Err(format!(
                    "Unsupported opretion '{:?}' at state '{:?}'.",
                    op, self.state
                ))
            }
        }

        Ok(())
    }

    fn operate_wait_for_rinshanhai(&mut self, op: &Operation) -> Result<(), String> {
        match op {
            Operation::Tehai(TehaiOperation::Add { hai, bound_check }) => {
                if let Err(error) = self.haiyama.discard(hai) {
                    if *bound_check {
                        return Err(error);
                    }
                }
                self.tehai.as_mut().unwrap().juntehai.push(*hai);
                self.tehai.as_mut().unwrap().juntehai.sort();
                self.state = State::FullHai;
            }
            Operation::Haiyama {
                kind: HaiyamaOperation::Add(hai_vec),
                bound_check,
            } => {
                if let Err(error) = self.haiyama.add_with_vec(hai_vec, *bound_check) {
                    if *bound_check {
                        return Err(error);
                    }
                }
            }
            Operation::Haiyama {
                kind: HaiyamaOperation::Discard(hai_vec),
                bound_check,
            } => {
                if let Err(error) = self.haiyama.discard_with_vec(hai_vec, *bound_check) {
                    if *bound_check {
                        return Err(error);
                    }
                }
            }
            _ => {
                return Err(format!(
                    "Unsupported opretion '{:?}' at state '{:?}'.",
                    op, self.state
                ))
            }
        }
        Ok(())
    }
}

impl std::fmt::Display for GameManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut sutehai_type_string = "".to_string();
        if self.sutehai_type.len() == 0 {
            sutehai_type_string += "無し";
        } else {
            for hai in self.sutehai_type.iter() {
                sutehai_type_string += &hai.to_string();
                sutehai_type_string += " ";
            }
        }

        write!(
            f,
            "牌山:\n  {}\n捨て牌の種類:\n  {}\n手牌:\n  {}\n状態:\n  {:?}",
            self.haiyama.to_string(),
            sutehai_type_string,
            match &self.tehai {
                Some(tehai) => tehai.to_string(),
                None => "Not initialized.".to_string(),
            },
            self.state
        )
    }
}
