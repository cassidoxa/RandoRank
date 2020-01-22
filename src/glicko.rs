use std::{collections::HashMap, f64::consts::PI as pi};

use chrono::NaiveDateTime;
use itertools::Itertools;
use pyo3::prelude::*;

use crate::{math, GlickoError};

#[derive(Clone, Copy, Debug)]
struct GlickoConstants {
    glicko_tau: f64,
    multi_slope: f64,
    multi_cutoff: u32,
    norm_factor: f64,
    initial_rating: f64,
    initial_deviation: f64,
    initial_volatility: f64,
}

impl Default for GlickoConstants {
    fn default() -> Self {
        GlickoConstants {
            glicko_tau: 0.2,
            multi_slope: 0.008,
            multi_cutoff: 8,
            norm_factor: 1.3,
            initial_rating: 1500.0,
            initial_deviation: 300.0,
            initial_volatility: 0.22,
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

    fn decay_score(&mut self, inactive_periods: u32) {
        let decayed_score: f64 =
            self.rating - ((self.deviation.ln().powi(2) + (inactive_periods as f64).sqrt()) / 2f64);
        self.rating = decayed_score;
    }
}

#[derive(Debug)]
struct Player {
    glicko_rating: GlickoRating,
    variance: f64,
    delta: f64,
    inactive_periods: u32,
    races: Vec<RaceResult>,
}

#[derive(Debug)]
struct Opponent {
    glicko_score: f64,
    normed_score: f64,
    rating: GlickoRating,
}

#[derive(Debug)]
struct RaceResult {
    datetime: Option<NaiveDateTime>,
    race_size: u32,
    player: (f64, f64), // (glicko_score, normed_score)
    opponent: Opponent,
}

#[pyclass]
pub struct MultiPeriod {
    players: HashMap<String, Player>,
    glicko_constants: GlickoConstants,
}

#[pymethods]
impl MultiPeriod {
    #[new]
    fn new(obj: &PyRawObject) {
        obj.init({
            MultiPeriod {
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

    fn set_initial_rating(&mut self, rating: f64) -> PyResult<()> {
        self.glicko_constants.initial_rating = rating;

        Ok(())
    }

    fn set_initial_deviation(&mut self, deviation: f64) -> PyResult<()> {
        self.glicko_constants.initial_deviation = deviation;

        Ok(())
    }

    fn set_initial_volatility(&mut self, vol: f64) -> PyResult<()> {
        self.glicko_constants.initial_volatility = vol;

        Ok(())
    }

    fn set_glicko_tau(&mut self, tau: f64) -> PyResult<()> {
        self.glicko_constants.glicko_tau = tau;

        Ok(())
    }

    fn set_norm_factor(&mut self, factor: f64) -> PyResult<()> {
        self.glicko_constants.norm_factor = factor;

        Ok(())
    }

    fn set_multi_slope(&mut self, slope: f64) -> PyResult<()> {
        self.glicko_constants.multi_slope = slope;

        Ok(())
    }

    fn set_multi_cutoff(&mut self, cutoff: f64) -> PyResult<()> {
        self.glicko_constants.multi_cutoff = cutoff as u32;

        Ok(())
    }

    #[cfg_attr(rustfmt, rustfmt_skip)]
    #[getter]
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

    #[getter]
    fn get_players(&self) -> Vec<&String> {
        let players: Vec<&String> = self.players.keys().collect();

        players
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
                inactive_periods: players[p]["inactive_periods"] as u32,
                races: Vec::with_capacity(20),
            };

            self.players.insert(p.to_string(), player);
        }

        Ok(())
    }

    fn add_race(&mut self, race: HashMap<String, f64>) -> PyResult<()> {
        validate_race(&race)?;
        self.add_new_players(&race)?;
        let num_finishers = race
            .values()
            .filter(|x| x.is_nan() == false)
            .map(|x| *x)
            .collect::<Vec<f64>>()
            .len();
        let normed_race = math::normalize_race(&race, &self.glicko_constants.norm_factor);
        self.make_pairings(&normed_race, num_finishers)?;

        Ok(())
    }

    #[args(end = true)]
    fn rank(&self, end: bool) -> PyResult<HashMap<&str, HashMap<&str, f64>>> {
        let mut rankings_dict: HashMap<&str, HashMap<&str, f64>> =
            HashMap::with_capacity(self.players.len());
        for (name, player) in self.players.iter() {
            if player.races.len() == 0 {
                // player hasn't raced. only change RD and apply
                let player_dict = self.process_inactive(player);
                rankings_dict.insert(name, player_dict);
            } else {
                // player has raced, process their 1v1s and add them to the
                // rankings hash map
                let player_dict = self.process_1v1s(player, end);
                rankings_dict.insert(name, player_dict);
            }
        }

        Ok(rankings_dict)
    }
}

// private methods not accessible from python
impl MultiPeriod {
    fn new_unrated(&mut self, name: &str) {
        let initial_glicko = GlickoRating {
            rating: self.glicko_constants.initial_rating,
            deviation: self.glicko_constants.initial_deviation,
            volatility: self.glicko_constants.initial_volatility,
        };
        let new_player = Player {
            glicko_rating: initial_glicko,
            variance: 0.0,
            delta: 0.0,
            inactive_periods: 0,
            races: Vec::with_capacity(20),
        };
        self.players.insert(name.to_string(), new_player);
    }

    fn add_new_players(&mut self, race: &HashMap<String, f64>) -> Result<(), GlickoError> {
        let new_racers: Vec<&String> = race
            .keys()
            .filter(|x| self.players.contains_key(x.as_str()) == false)
            .collect();
        if new_racers.is_empty() == false {
            new_racers.iter().for_each(|x| self.new_unrated(x));
        }

        Ok(())
    }

    fn make_pairings(
        &mut self,
        race: &HashMap<String, (f64, f64)>,
        num_finishers: usize,
    ) -> Result<(), GlickoError> {
        let players: Vec<&String> = race.keys().collect();
        let perms = players.iter().permutations(2);
        let score = |p: f64, o: f64| -> f64 {
            if p < o {
                1f64
            } else if p > o {
                0f64
            } else {
                0.5f64
            }
        };

        for pair in perms {
            let opponent = Opponent {
                glicko_score: score(race[*pair[1]].0, race[*pair[0]].0),
                normed_score: race[*pair[1]].1,
                rating: self.players[*pair[1]].glicko_rating,
            };
            let race_result = RaceResult {
                datetime: None,
                race_size: num_finishers as u32,
                player: (score(race[*pair[0]].0, race[*pair[1]].0), race[*pair[0]].1),
                opponent: opponent,
            };
            self.players
                .entry(pair[0].to_string())
                .and_modify(|x| x.races.push(race_result));
        }
        Ok(())
    }

    fn process_1v1s(&self, player: &Player, end: bool) -> HashMap<&str, f64> {
        let mut player_dict: HashMap<&str, f64> = HashMap::with_capacity(6);
        let initial_rating = self.glicko_constants.initial_rating;
        let mut converted_rating = player.glicko_rating.convert_to(initial_rating);
        let mut v_inv = sanitize_v(player.variance.recip());
        let mut delta = player.delta;
        let tau = self.glicko_constants.glicko_tau;

        for r in &player.races {
            let ndiff: f64 = (r.player.1 - r.opponent.normed_score).abs();
            let size = r.race_size;
            let slope = self.glicko_constants.multi_slope;
            let opp = r.opponent.rating.convert_to(initial_rating);
            let multi_factor =
                (1f64 - (slope * (size as f64).powf(1f64 - ndiff))) * (1f64 / (1f64 - slope));
            let mut weight = 1f64 / (1f64 + (3f64 * opp.deviation.powi(2) / pi.powi(2))).sqrt();
            if size >= self.glicko_constants.multi_cutoff {
                weight = weight * multi_factor;
            }
            let expected_score =
                1f64 / (1f64 + (-weight * (converted_rating.rating - opp.rating)).exp());
            v_inv += weight.powi(2) * expected_score * (1f64 - expected_score);
            delta += weight * (r.player.0 as f64 - expected_score);
        }
        if v_inv != 0f64 {
            let var = 1f64 / v_inv;
            let change = var * delta;
            let new_sigma = math::get_sigma(
                tau,
                converted_rating.deviation,
                converted_rating.volatility,
                change,
                var,
            );
            let phi_star: f64 = (converted_rating.deviation.powi(2) + new_sigma.powi(2)).sqrt();
            converted_rating.deviation = 1f64 / ((1f64 / phi_star.powi(2)) + (1f64 / var)).sqrt();
            converted_rating.rating =
                converted_rating.rating + converted_rating.deviation.powi(2) * delta;
            converted_rating.volatility = new_sigma;
            let new_rating = converted_rating.convert_from(initial_rating);

            player_dict.insert("rating", new_rating.rating);
            player_dict.insert("deviation", new_rating.deviation);
            player_dict.insert("inactive_periods", 0f64);
            if end {
                player_dict.insert("volatility", new_rating.volatility);
                player_dict.insert("variance", 0f64);
                player_dict.insert("delta", 0f64);
            } else {
                player_dict.insert("volatility", player.glicko_rating.volatility);
                player_dict.insert("variance", var);
                player_dict.insert("delta", delta);
            }
        } else {
            let var = 0f64;
            let new_rating = converted_rating.convert_from(initial_rating);

            player_dict.insert("rating", new_rating.rating);
            player_dict.insert("deviation", new_rating.deviation);
            player_dict.insert("volatility", new_rating.volatility);
            player_dict.insert("inactive_periods", 0f64);
            if end {
                player_dict.insert("volatility", new_rating.volatility);
                player_dict.insert("variance", 0f64);
                player_dict.insert("delta", 0f64);
            } else {
                player_dict.insert("volatility", player.glicko_rating.volatility);
                player_dict.insert("variance", var);
                player_dict.insert("delta", delta);
            }
        }

        player_dict
    }

    fn process_inactive(&self, player: &Player) -> HashMap<&str, f64> {
        let mut player_dict: HashMap<&str, f64> = HashMap::with_capacity(6);
        let initial_rating = self.glicko_constants.initial_rating;
        let mut converted_rating = player.glicko_rating.convert_to(initial_rating);
        let inactive_periods: u32 = player.inactive_periods + 1;
        let phi_star: f64 =
            (converted_rating.deviation.powi(2) + converted_rating.volatility.powi(2)).sqrt();
        converted_rating.deviation = phi_star;
        let mut new_rating = converted_rating.convert_from(initial_rating);
        if player.inactive_periods > 0 {
            new_rating.decay_score(player.inactive_periods);
        }
        player_dict.insert("rating", new_rating.rating);
        player_dict.insert("deviation", new_rating.deviation);
        player_dict.insert("volatility", new_rating.volatility);
        player_dict.insert("variance", 0f64);
        player_dict.insert("delta", 0f64);
        player_dict.insert("inactive_periods", inactive_periods as f64);

        player_dict
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
    const REQUIRED_KEYS: [&str; 6] = [
        "rating",
        "deviation",
        "volatility",
        "variance",
        "delta",
        "inactive_periods",
    ];

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

fn validate_race(race: &HashMap<String, f64>) -> PyResult<()> {
    // confirm that the race has:
    // 1. At least two players
    // 2. At least one non-forfeiting player

    if race.len() < 2 {
        return Err(GlickoError::py_err(
            "Invalid race passed to method: Less than two racers",
        ));
    }

    let times: Vec<&f64> = race.values().filter(|x| x.is_nan() == false).collect();
    if times.len() < 1 {
        return Err(GlickoError::py_err(
            "Invalid race passed to method: Less than one finisher",
        ));
    }

    Ok(())
}

fn sanitize_v(v: f64) -> f64 {
    if v.is_finite() {
        v
    } else {
        0f64
    }
}
