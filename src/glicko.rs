use std::{collections::HashMap, f64::NAN as NaN};

use chrono::NaiveDateTime;
use pyo3::prelude::*;

use crate::{math, GlickoError};

const GLICKO_CONVERSION: f64 = 173.7178;
const _WIN: f64 = 1.0;
const _DRAW: f64 = 0.5;
const _LOSS: f64 = 0.0;
const EPSILON: f64 = 0.000_000_001;

struct GlickoConstants {
    glicko_tau: f64,
    multi_slope: f64,
    multi_cutoff: u32,
    norm_factor: f64,
    initial_rating: f64,
    initial_deviation: f64,
    initial_volatility: f64,
}

impl GlickoConstants {
    fn unrated(&self) -> GlickoRating {
        GlickoRating {
            rating: self.initial_rating,
            deviation: self.initial_deviation,
            volatility: self.initial_volatility,
        }
    }
}

impl Default for GlickoConstants {
    fn default() -> Self {
        GlickoConstants {
            glicko_tau: 0.2,
            multi_slope: 0.0008,
            multi_cutoff: 8,
            norm_factor: 1.3,
            initial_rating: 1500.0,
            initial_deviation: 50.0,
            initial_volatility: 0.01,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct GlickoRating {
    rating: f64,
    deviation: f64,
    volatility: f64,
}

impl GlickoRating {
    fn convert_from(&self, initial_rating: f64) -> GlickoRating {
        GlickoRating {
            rating: (self.rating * 173.7178) + initial_rating,
            deviation: self.deviation * 173.7178,
            volatility: self.volatility,
        }
    }

    fn convert_to(&self, initial_rating: f64) -> GlickoRating {
        GlickoRating {
            rating: (self.rating - initial_rating) / 173.7178,
            deviation: self.deviation / 173.7178,
            volatility: self.volatility,
        }
    }
}

struct Player {
    glicko_rating: GlickoRating,
    variance: f64,
    delta: f64,
    races: Vec<RaceResult>,
}

struct Opponent {
    time: u32,
    score: f64,
    rating: GlickoRating,
}

// struct MultiRace<'a> {
//     num_finishers: u32,
//     datetime: Option<NaiveDateTime>,
//     results: HashMap<&'a str, u32>,
// }

struct RaceResult {
    datetime: Option<NaiveDateTime>,
    player: (u32, f64),
    oppenent: Opponent,
}

#[pyclass(module = "randorank")]
pub struct Period {
    players: HashMap<String, Player>,
    glicko_constants: GlickoConstants,
}

#[pymethods]
impl Period {
    #[new]
    fn new(obj: &PyRawObject) {
        obj.init({
            Period {
                players: HashMap::with_capacity(100),
                glicko_constants: GlickoConstants::default(),
            }
        })
    }

    fn set_constants(&mut self, constants: HashMap<&str, f64>) -> PyResult<()> {
        validate_constants(&constants)?;
        let new_constants: GlickoConstants = GlickoConstants {
            glicko_tau: constants["tau"],
            multi_slope: constants["multi_slope"],
            multi_cutoff: constants["multi_cutoff"] as u32,
            norm_factor: constants["norm_factor"],
            initial_rating: constants["initial_rating"],
            initial_deviation: constants["initial_deviation"],
            initial_volatility: constants["initial_volatility"],
        };
        self.glicko_constants = new_constants;

        Ok(())
    }

    #[getter(constants)]
    #[cfg_attr(rustfmt, rustfmt_skip)]
    fn get_constants(&self) -> PyResult<HashMap<&str, f64>> {
        let mut constants: HashMap<&str, f64> = HashMap::with_capacity(6);
        constants.insert("tau", self.glicko_constants.glicko_tau);
        constants.insert("multi_slope", self.glicko_constants.multi_slope);
        constants.insert("multi_cutoff", self.glicko_constants.multi_cutoff as f64);
        constants.insert("norm_factor", self.glicko_constants.norm_factor);
        constants.insert("initial_rating", self.glicko_constants.initial_rating);
        constants.insert("initial_deviation", self.glicko_constants.initial_deviation);
        constants.insert("initial_volatility", self.glicko_constants.initial_volatility,);

        Ok(constants)
    }

    fn add_players(&mut self, players: HashMap<String, HashMap<String, f64>>) -> PyResult<()> {
        validate_players(&players)?;
        for p in players.keys() {
            let glicko = GlickoRating {
                rating: players[p]["rating"],
                deviation: players[p]["deviation"],
                volatility: players[p]["volatility"],
            };
            let player = Player {
                glicko_rating: glicko,
                variance: players[p]["variance"],
                delta: players[p]["delta"],
                races: Vec::with_capacity(20),
            };

            self.players.insert(p.to_string(), player);
        }

        Ok(())
    }
    // take one race, split into 1v1s,
    fn add_race(&mut self, race: HashMap<String, Option<u32>>) -> PyResult<()> {
        validate_race(&race)?;
        // add any new racers with default values
        self.add_new_players(&race)?;
        let normed_race = math::normalize_race(&race, &self.glicko_constants.norm_factor);
        // create 1v1s with time and normed score, add to Vec owned by player struct
        Ok(())
    }
    //fn rank_players(&self) -> HashMap<HashMap<&str, f64>>{}
}

// private methods not accessible from python
impl Period {
    fn new_unrated(&mut self, name: &str) {
        let initial_glicko = GlickoRating {
            rating: self.glicko_constants.initial_rating,
            deviation: self.glicko_constants.initial_deviation,
            volatility: self.glicko_constants.initial_volatility,
        };
        let new_player = Player {
            glicko_rating: self.glicko_constants.unrated(),
            variance: 0.0,
            delta: 0.0,
            races: Vec::with_capacity(20),
        };
        self.players.insert(name.to_string(), new_player);
    }

    fn add_new_players(&mut self, race: &HashMap<String, Option<u32>>) -> Result<(), GlickoError> {
        let new_racers: Vec<&String> = race
            .keys()
            .filter(|x| self.players.contains_key(x.as_str()) == false)
            .collect();
        if new_racers.is_empty() == false {
            new_racers.iter().for_each(|x| self.new_unrated(x));
        }

        Ok(())
    }
}

fn validate_constants(constants: &HashMap<&str, f64>) -> PyResult<()> {
    const REQUIRED_CONSTANTS: [&str; 7] = [
        "tau",
        "multi_slope",
        "multi_cutoff",
        "norm_factor",
        "initial_rating",
        "initial_deviation",
        "initial_volatility",
    ];
    if REQUIRED_CONSTANTS
        .iter()
        .all(|&k| constants.contains_key(k))
        == false
    {
        return Err(GlickoError::py_err(
            "Not all Glicko constants found in dict",
        ));
    }
    if constants.keys().all(|x| REQUIRED_CONSTANTS.contains(x)) == false {
        return Err(GlickoError::py_err(
            "Malformed constants dict passed to method",
        ));
    }

    Ok(())
}

fn validate_players(players: &HashMap<String, HashMap<String, f64>>) -> PyResult<()> {
    const REQUIRED_KEYS: [&str; 5] = ["rating", "deviation", "volatility", "variance", "delta"];

    for m in players.values() {
        if REQUIRED_KEYS.iter().all(|&k| m.contains_key(k)) == false {
            return Err(GlickoError::py_err(
                "Not all player attributes found in dict",
            ));
        }
        if m.keys().all(|x| REQUIRED_KEYS.contains(&x.as_str())) == false {
            return Err(GlickoError::py_err(
                "Malformed player dict passed to method",
            ));
        }
    }

    Ok(())
}

fn validate_race(race: &HashMap<String, Option<u32>>) -> PyResult<()> {
    // confirm that the race has:
    // 1. At least two players
    // 2. At least one non-forfeiting player

    if race.len() < 2 {
        return Err(GlickoError::py_err(
            "Invalid race passed to method: Less than two racers",
        ));
    }

    let times: Vec<u32> = race
        .values()
        .filter(|x| x.is_some())
        .map(|&x| x.unwrap())
        .collect();
    if times.len() < 1 {
        return Err(GlickoError::py_err(
            "Invalid race passed to method: Less than one finisher",
        ));
    }

    Ok(())
}
