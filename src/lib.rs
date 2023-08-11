use std::num::NonZeroU8;

use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;
use rand::Rng;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Board {
    board: [[Option<NonZeroU8>; 4]; 4],
}

impl Board {
    pub fn new(rng: &mut ThreadRng) -> Self {
        let mut initial_board = [[None; 4]; 4];
        let indice = (0..4)
            .map(|i| (0..4).map(move |j| (i, j)))
            .flatten()
            .collect::<Vec<_>>();
        let posi = indice.choose_multiple(rng, 2);
        posi.for_each(|&(x, y)| initial_board[x][y] = NonZeroU8::new(1));
        initial_board.into()
    }

    pub fn play(&mut self, direction: Arrow, rng: &mut ThreadRng) -> bool {
        self.merge(direction);
        self.gen_num(rng);
        self.is_lost()
    }

    pub fn gen_num(&mut self, rng: &mut ThreadRng) -> bool {
        if self.is_full() {
            return false;
        }

        let &(x, y) = (0..4)
            .map(|i| (0..4).map(move |j| (i, j)))
            .flatten()
            .filter(|&(x, y)| self.board[x][y].is_none())
            .collect::<Vec<_>>()
            .choose(rng)
            .unwrap();

        self.board[x][y] = if rng.gen_ratio(1, 10) {
            NonZeroU8::new(2)
        } else {
            NonZeroU8::new(1)
        };

        true
    }

    fn is_full(&self) -> bool {
        self.board
            .iter()
            .map(|row| row.iter())
            .flatten()
            .all(Option::is_some)
    }

    pub fn is_lost(&self) -> bool {
        self.is_full() && !self.is_mergable()
    }
}

impl Board {
    fn is_mergable(&self) -> bool {
        let mergable_row = || {
            (0..4).any(|x| {
                (0..3).map(|y| (x, y)).any(|(x, y)| {
                    let left = self.board[x][y];
                    let right = self.board[x][y + 1];
                    left.is_some() && left == right
                })
            })
        };
        let mergable_col = || {
            (0..3).any(|x| {
                (0..4).map(|y| (x, y)).any(|(x, y)| {
                    let above = self.board[x][y];
                    let below = self.board[x + 1][y];
                    above.is_some() && above == below
                })
            })
        };
        mergable_row() || mergable_col()
    }

    fn scan(
        &mut self,
        direction: Arrow,
        op: impl Fn(&mut Option<NonZeroU8>, &mut Option<NonZeroU8>),
    ) {
        match direction {
            Arrow::Up => (0..3).rev().for_each(|x| {
                (0..4).map(|y| (x, y)).for_each(|(x, y)| {
                    let (above, below) = self.board.split_at_mut(x + 1);
                    let (above, below) = (
                        &mut above.last_mut().unwrap()[y],
                        &mut below.first_mut().unwrap()[y],
                    );
                    op(above, below);
                })
            }),
            Arrow::Down => (0..3).for_each(|x| {
                (0..4).map(|y| (x, y)).for_each(|(x, y)| {
                    let (above, below) = self.board.split_at_mut(x + 1);
                    let (above, below) = (
                        &mut above.last_mut().unwrap()[y],
                        &mut below.first_mut().unwrap()[y],
                    );
                    op(above, below);
                })
            }),
            Arrow::Left => (0..4).for_each(|x| {
                (0..3).rev().map(|y| (x, y)).for_each(|(x, y)| {
                    let (left, right) = self.board[x].split_at_mut(y + 1);
                    let (left, right) = (left.last_mut().unwrap(), right.first_mut().unwrap());
                    op(left, right);
                })
            }),
            Arrow::Right => (0..4).for_each(|x| {
                (0..3).rev().map(|y| (x, y)).for_each(|(x, y)| {
                    let (left, right) = self.board[x].split_at_mut(y + 1);
                    let (left, right) = (left.last_mut().unwrap(), right.first_mut().unwrap());
                    op(left, right);
                })
            }),
        }
    }

    fn merge(&mut self, direction: Arrow) {
        self.squash(direction);

        match direction {
            Arrow::Up => self.scan(direction, |above, below| {
                if above.is_some() && above == below {
                    *below = above.unwrap().checked_add(1);
                    *above = None;
                }
            }),
            Arrow::Down => self.scan(direction, |above, below| {
                if above.is_some() && above == below {
                    *above = below.unwrap().checked_add(1);
                    *below = None;
                }
            }),
            Arrow::Left => self.scan(direction, |left, right| {
                if right.is_some() && left == right {
                    *right = left.unwrap().checked_add(1);
                    *left = None;
                }
            }),
            Arrow::Right => self.scan(direction, |left, right| {
                if right.is_some() && left == right {
                    *left = right.unwrap().checked_add(1);
                    *right = None;
                }
            }),
        }

        self.squash(direction);
    }

    fn squash_once(&mut self, direction: Arrow) {
        match direction {
            Arrow::Up => self.scan(direction, |above, below| {
                if above.is_none() && below.is_some() {
                    *above = below.take();
                }
            }),
            Arrow::Down => self.scan(direction, |above, below| {
                if below.is_none() && above.is_some() {
                    *below = above.take();
                }
            }),
            Arrow::Left => self.scan(direction, |left, right| {
                if left.is_none() && right.is_some() {
                    *left = right.take();
                }
            }),
            Arrow::Right => self.scan(direction, |left, right| {
                if right.is_none() && left.is_some() {
                    *right = left.take();
                }
            }),
        }
    }

    fn squash(&mut self, direction: Arrow) {
        for _ in 0..3 {
            self.squash_once(direction);
        }
    }
}

impl From<[[Option<NonZeroU8>; 4]; 4]> for Board {
    fn from(value: [[Option<NonZeroU8>; 4]; 4]) -> Self {
        Self { board: value }
    }
}

mod tests {
    #![allow(unused_imports)]
    use super::*;

    #[test]
    fn test_mergable() {
        let mergable_boards = [
            [
                [None; 4],
                [None; 4],
                [None, None, NonZeroU8::new(3), None],
                [None, None, NonZeroU8::new(3), None],
            ],
            [
                [None; 4],
                [None; 4],
                [None; 4],
                [None, None, NonZeroU8::new(1), NonZeroU8::new(1)],
            ],
        ];

        let unmergable_boards = [
            [
                [None; 4],
                [None; 4],
                [None, None, NonZeroU8::new(2), None],
                [None, None, NonZeroU8::new(3), None],
            ],
            [
                [None; 4],
                [None; 4],
                [None; 4],
                [None, None, NonZeroU8::new(2), NonZeroU8::new(1)],
            ],
            [[None; 4], [None; 4], [None; 4], [None; 4]],
        ];
        assert!(mergable_boards
            .into_iter()
            .all(|board| Board::from(board).is_mergable()));

        assert!(unmergable_boards
            .into_iter()
            .all(|board| !Board::from(board).is_mergable()));
    }

    #[test]
    fn test_is_lost() {
        let lost_boards = [[
            [
                NonZeroU8::new(1),
                NonZeroU8::new(2),
                NonZeroU8::new(1),
                NonZeroU8::new(2),
            ],
            [
                NonZeroU8::new(2),
                NonZeroU8::new(1),
                NonZeroU8::new(2),
                NonZeroU8::new(1),
            ],
            [
                NonZeroU8::new(1),
                NonZeroU8::new(2),
                NonZeroU8::new(1),
                NonZeroU8::new(2),
            ],
            [
                NonZeroU8::new(2),
                NonZeroU8::new(1),
                NonZeroU8::new(2),
                NonZeroU8::new(1),
            ],
        ]];

        let not_yet_losts = [
            [
                [
                    NonZeroU8::new(1),
                    NonZeroU8::new(2),
                    NonZeroU8::new(1),
                    NonZeroU8::new(2),
                ],
                [
                    NonZeroU8::new(2),
                    NonZeroU8::new(1),
                    NonZeroU8::new(2),
                    NonZeroU8::new(1),
                ],
                [
                    NonZeroU8::new(1),
                    NonZeroU8::new(2),
                    NonZeroU8::new(1),
                    NonZeroU8::new(2),
                ],
                [
                    NonZeroU8::new(2),
                    NonZeroU8::new(1),
                    NonZeroU8::new(2),
                    None,
                ],
            ],
            [
                [
                    NonZeroU8::new(1),
                    NonZeroU8::new(2),
                    NonZeroU8::new(1),
                    NonZeroU8::new(2),
                ],
                [
                    NonZeroU8::new(3),
                    NonZeroU8::new(1),
                    NonZeroU8::new(2),
                    NonZeroU8::new(1),
                ],
                [
                    NonZeroU8::new(2),
                    NonZeroU8::new(2),
                    NonZeroU8::new(1),
                    NonZeroU8::new(2),
                ],
                [
                    NonZeroU8::new(2),
                    NonZeroU8::new(1),
                    NonZeroU8::new(2),
                    NonZeroU8::new(1),
                ],
            ],
        ];

        assert!(lost_boards
            .into_iter()
            .map(Board::from)
            .all(|board| board.is_lost()));
        assert!(not_yet_losts
            .into_iter()
            .map(Board::from)
            .all(|board| !board.is_lost()));
    }

    #[test]
    fn test_merge_squash() {
        let pairs = [
            (
                [
                    [None, None, None, NonZeroU8::new(1)],
                    [None; 4],
                    [None, None, NonZeroU8::new(3), None],
                    [None, None, NonZeroU8::new(3), NonZeroU8::new(1)],
                ],
                Arrow::Down,
                [
                    [None; 4],
                    [None; 4],
                    [None; 4],
                    [None, None, NonZeroU8::new(4), NonZeroU8::new(2)],
                ],
            ),
            (
                [[None; 4], [None; 4], [None; 4], [NonZeroU8::new(4); 4]],
                Arrow::Right,
                [
                    [None; 4],
                    [None; 4],
                    [None; 4],
                    [None, None, NonZeroU8::new(5), NonZeroU8::new(5)],
                ],
            ),
        ];
        assert!(pairs
            .into_iter()
            .map(|(left, op, right)| (Board::from(left), op, Board::from(right)))
            .all(|(mut left, op, right)| {
                left.merge(op);
                left == right
            }));
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Arrow {
    Up,
    Down,
    Left,
    Right,
}
