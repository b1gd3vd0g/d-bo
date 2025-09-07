use std::{collections::VecDeque, fmt::Debug};

use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Deserialize, Serialize)]
pub enum Card {
    Number(u8),
    DBo,
}

impl Debug for Card {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Card::Number(n) => write!(f, "{}", n),
            Card::DBo => write!(f, "D-Bo"),
        }
    }
}

pub struct PlayerState {
    pub player_id: String,
    pub hand: Vec<Card>,
    pub discard_piles: [VecDeque<Card>; 4],
    pub stock_pile: VecDeque<Card>,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Game {
    game_id: String,
    players: Vec<String>,
    pub deck: VecDeque<Card>,
}

impl Game {
    pub fn init(players: Vec<String>) -> Self {
        Self {
            game_id: Uuid::new_v4().to_string(),
            players: players,
            deck: {
                let mut deck = Self::unshuffled_deck();
                Self::shuffle(&mut deck);
                deck
            },
        }
    }

    fn unshuffled_deck() -> VecDeque<Card> {
        let mut deck: VecDeque<Card> = VecDeque::new();

        for i in 1..=12 {
            for _ in 0..12 {
                deck.push_back(Card::Number(i));
            }
        }

        for _ in 0..18 {
            deck.push_back(Card::DBo);
        }

        deck
    }

    fn shuffle(cards: &mut VecDeque<Card>) {
        let mut v: Vec<Card> = cards.drain(..).collect();
        v.shuffle(&mut rand::rng());
        *cards = v.into();
    }
}
