use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{poll, read, Event, KeyCode},
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{enable_raw_mode, size, Clear, ClearType},
    ExecutableCommand, QueueableCommand,
};
use std::{
    borrow::{Borrow, BorrowMut}, clone, default, io::{stdout, Result, Stdout, Write}, ops::{Deref, DerefMut}, thread::{self, sleep}, vec
};

use rodio::{buffer, source::Source, Decoder, OutputStream};
use std::fs::File;
use std::io::BufReader;

use inline_colorization::*;
use ndarray::{Array, Array2};
use rand::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::Duration;
// use std::thread;

/*
** GAME PHASES
* keyboard: binding the keyboard to listening to the keys that inserted
* physics: performing the physical changes for the boats or the enemies
* drawing: performing the changes related to the screen.
*/
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub enum GameStatus {
    #[default]
    ALIVE,
    DEATH, /*PAUSED, FUEL_ENDED*/
}

#[derive(Clone, Debug)]
pub struct Location {
    pub element_i: u16,
    pub element_j: u16,
}
#[derive(Clone, Debug)]
pub struct Enemy {
    pub location: Location,
    pub logo: String,
}
#[derive(Clone, Debug)]
pub struct Bullet {
    pub location: Location,
    pub active: bool,
    pub logo: String,
}

#[derive(Clone, Debug)]
pub struct Fuel {
    pub location: Location,
    pub logo: String,
}

pub enum Sound {
    EnemyKilled(String),
    FuelObtained(String),
    BoatCrashed(String),
}

#[derive(Clone, Debug, Default)]
pub struct Game2DMatrix {
    pub player_i: u16,
    pub player_j: u16,
    pub max_screen_i: u16,
    pub max_screen_j: u16,
    pub screen_mid: u16,
    pub map: Array2<f64>,
    pub ground: Vec<(u16, u16)>,
    pub enemies: Vec<Enemy>,
    pub bullets: Vec<Bullet>,
    pub fuels: Vec<Fuel>,
    pub game_staus: GameStatus,
    pub score: u32,
    pub gas: u32,
    pub enemy_killed: u32,
    pub initialized: bool,
    pub logo: String,
}

impl Game2DMatrix {
    // NOTE: implementing Defaul trait for Game2DMatrix structure.
    pub fn new() -> Self {
        // initialize the game information
        let (max_i, max_j) = size().unwrap();

        Game2DMatrix {
            player_i: max_i / 2,
            player_j: max_j - 10,
            max_screen_i: max_i,
            max_screen_j: max_j,
            screen_mid: max_i / 2,
            map: Array::from_shape_vec(
                (max_i as usize, max_j as usize),
                vec![0.0; (max_i * max_j) as usize],
            )
            .unwrap(),
            ground: vec![(0, 0); max_j as usize],
            enemies: Vec::new(),
            bullets: Vec::new(),
            fuels: Vec::new(),
            game_staus: GameStatus::ALIVE,
            score: 0,
            gas: 1500,
            enemy_killed: 0,
            initialized: false,
            logo: '⛵'.to_string(),
        }
    }

    /// @notice this function will use at the beginning of the game to initialize the ground borders.
    /// @dev this function will use in the draw function is self.initialize was false.
    pub fn initialize_ground(&mut self, screen: &mut Stdout) -> Result<()> {
        // initial phase of screen
        screen.queue(Clear(ClearType::All))?;

        let mut rng = rand::thread_rng();
        // let screen_mid = self.max_screen_i / 2;
        let mut lg_range: u16;
        let mut rg_range: u16;
        let mut change_precision = rng.gen_range(5..10);

        for i in 0..self.map.row(0).len() {
            if i % change_precision == 0 {
                lg_range = rng.gen_range((self.screen_mid - 40)..self.screen_mid);
                rg_range = rng.gen_range(self.screen_mid..(self.screen_mid + 40));

                (self.ground[i].0, self.ground[i].1) = (lg_range, rg_range);
                change_precision = rng.gen_range(change_precision..15);
            } else {
                (self.ground[i].0, self.ground[i].1) = (self.ground[i - 1].0, self.ground[i - 1].1);
            }
        }

        self.initialized = true;
        Ok(())
    }

    pub fn draw(
        &mut self,
        screen: &mut Stdout,
        show_enemy: bool,
        show_fuel: bool,
    ) -> Result<&mut Self> {
        screen.queue(Clear(ClearType::All))?;

        // draw the map as first scence
        for j in 0..(self.map.row(0).len()) {
            screen
                .queue(MoveTo(0, j as u16))? // (i, j)
                .queue(SetForegroundColor(Color::Green))?
                .queue(SetBackgroundColor(Color::Green))?
                .queue(Print(" ".repeat(self.ground[j].0 as usize)))?
                .queue(MoveTo(self.ground[j].0, j as u16))?
                .queue(SetBackgroundColor(Color::Blue))?
                .queue(Print(
                    " ".repeat((self.ground[j].1 - self.ground[j].0) as usize),
                ))?
                .queue(MoveTo(self.ground[j].1, j as u16))?
                .queue(SetBackgroundColor(Color::Green))?
                .queue(Print(
                    " ".repeat((self.max_screen_i - self.ground[j].1) as usize),
                ))?
                .queue(ResetColor)?;
        }

        for bullet in self.bullets.iter() {
            screen
                .queue(MoveTo(bullet.location.element_j, bullet.location.element_i))?
                .queue(SetBackgroundColor(Color::Blue))?
                .queue(Print(&bullet.logo))?;
        }

        // adjust furl in the posibility of 10% of situations.
        if show_fuel {
            self.fuels.push(Fuel {
                location: Location {
                    element_i: 2,
                    element_j: rand::thread_rng().gen_range(self.ground[2].0..self.ground[2].1),
                },
                logo: '⛽'.to_string(),
            })
        }

        for fuel in self.fuels.iter() {
            screen
                .queue(MoveTo(fuel.location.element_j, fuel.location.element_i))?
                .queue(SetBackgroundColor(Color::Blue))?
                .queue(Print(&fuel.logo))?;
        }

        // adjust enemy in the posibility of 10% of situations
        if show_enemy {
            self.enemies.push(Enemy {
                location: Location {
                    element_i: 2,
                    element_j: rand::thread_rng().gen_range(self.ground[2].0..self.ground[2].1),
                },
                logo: '👾'.to_string(),
            })
        }

        for enemy in self.enemies.iter() {
            screen
                .queue(MoveTo(enemy.location.element_j, enemy.location.element_i))?
                .queue(SetBackgroundColor(Color::Blue))?
                .queue(Print(&enemy.logo))?
                .queue(ResetColor)?;
        }

        // draw the player
        screen
            .queue(MoveTo(self.player_i, self.player_j))?
            .queue(SetBackgroundColor(Color::Blue))?
            .queue(Print(&self.logo))?;

        // draw the game scores and status
        let scores_position = (self.max_screen_i / 13, self.max_screen_j / 13);

        screen
            .queue(SetBackgroundColor(Color::DarkGrey))?
            .queue(MoveTo(scores_position.0, scores_position.1))?
            .queue(Print(format!("Score: {}", self.score)))?
            .queue(SetBackgroundColor(Color::DarkGrey))?
            .queue(MoveTo(scores_position.0, scores_position.1 + 1))?
            .queue(Print(format!("Enemy killed: {}", self.enemy_killed)))?
            .queue(SetBackgroundColor(Color::DarkGrey))?
            .queue(MoveTo(scores_position.0, scores_position.1 + 2))?
            .queue(Print(format!("Fuel: {}", self.gas)))?
            .queue(SetBackgroundColor(Color::DarkGrey))?
            .queue(ResetColor)?;

        screen.flush()?;
        Ok(self)
    }

    /// @notice this function perform the elements' movement during the game loop i.e. bullets, enemies, etc.
    /// @dev this function will be called after the draw function to get the modified nd2array game information.
    pub fn shift_ground_loc(&mut self, change: bool) -> Result<&mut Self> {
        for i in (1..self.map.row(0).len()).rev() {
            self.ground[i] = self.ground[i - 1];
        }

        for bullet in self.bullets.iter_mut().rev() {
            bullet.location.element_i = bullet.location.element_i.saturating_sub(2);
        }
        self.bullets.retain(|bullet| bullet.location.element_i >= 3);

        // manipulate the existing enemies in the map.
        for enemy in self.enemies.iter_mut() {
            enemy.location.element_i = enemy.location.element_i.saturating_add(1);
        }

        for fuel in self.fuels.iter_mut() {
            fuel.location.element_i = fuel.location.element_i.saturating_add(1);
        }

        let mut rng = rand::thread_rng();
        let delta = rng.gen_range(1..6);

        self.score += 1;
        if self.score % 2 == 0 {
            self.gas -= 1;
        }

        if change && (self.ground[1].1 < self.max_screen_i - 5) {
            self.ground[0] = (self.ground[1].0 + delta, self.ground[1].1 + delta);
        } else if self.ground[1].0 > delta {
            self.ground[0] = (self.ground[1].0 - delta, self.ground[1].1 - delta);
        } else {
            self.ground[0] = self.ground[1];
        }

        Ok(self)
    }


    pub fn reactions(&mut self /*screen: &mut Stdout*/) -> Result<()> {
        let user_j: usize = self.player_j as usize;

        if self.gas == 0 {
            self.game_staus = GameStatus::DEATH;
        }

        // handling the boat accidentation with ground
        if self.player_i <= self.ground[user_j].0 || self.player_i >= self.ground[user_j].1 {
            self.game_staus = GameStatus::DEATH;
        }

        /////////////////////////////// Take reaction to the enemies chars. ///////////////////////////////
        let mut enemies_to_remove: Vec<usize> = vec![];

        for (idx, enemy) in self.enemies.iter_mut().enumerate() {
            // player collision with the enemies in the ground.
            if (enemy.location.element_j - 1..enemy.location.element_j + 1).contains(&self.player_i)
                && enemy.location.element_i == self.player_j
            {
                self.game_staus = GameStatus::DEATH;
            }

            // the reaction related to the player's bullets verses the enemies.
            for bullet in self.bullets.iter_mut() {
                if bullet.active
                    && (enemy.location.element_i - 2..enemy.location.element_i + 2)
                        .contains(&bullet.location.element_i)
                    && (enemy.location.element_j - 2..enemy.location.element_j + 2)
                        .contains(&bullet.location.element_j)
                {
                    std::thread::spawn(move || {
                        handle_sound("src/assets/demon-death.wav".to_string(), 1.5);
                    });

                    enemy.logo = ' '.to_string();
                    enemies_to_remove.push(idx);
                    sleep(Duration::from_millis(100));
                    bullet.active = false;
                    bullet.logo = ' '.to_string();
                }
            }
        }

        enemies_to_remove.sort_unstable_by(|a, b| b.cmp(a)); // Sort in reverse order
        for idx in enemies_to_remove {
            if idx < self.enemies.len() {
                self.enemies.remove(idx);
            }
            self.enemy_killed += 1;
        }

        /////////////////////////////// Take reaction to the fuel chars. ///////////////////////////////
        for fuel in self.fuels.iter() {
            if (fuel.location.element_j - 2..fuel.location.element_j + 2).contains(&self.player_i)
                && (fuel.location.element_i == self.player_j)
            {
                self.gas += 30;
            }
        }

        /////////////////////////////// Take reaction to bottom of the screen ///////////////////////////////
        self.enemies
            .retain(|enemy| enemy.location.element_i < self.max_screen_j - 3);

        self.fuels.retain(|fuel| {
            !((fuel.location.element_j - 1..fuel.location.element_j + 1).contains(&self.player_i)
                && (fuel.location.element_i == self.player_j)
                || (fuel.location.element_i > self.max_screen_j - 3))
        });
        Ok(())
    }

    pub fn multi_reactions(&'static mut self) -> Result<&mut Self> {
        let user_j: usize = self.player_j as usize;

        let arc_game = Arc::new(Mutex::new(self.clone()));
        let cloned_game = Arc::clone(&arc_game);

        let handle_accidents = thread::spawn(move || {
            let mut game = cloned_game.lock().unwrap();

            if game.gas == 0 {
                game.game_staus = GameStatus::DEATH;
            }

            // handling the boat accidentation with ground
            if game.player_i <= game.ground[user_j].0 || game.player_i >= game.ground[user_j].1 {
                game.game_staus = GameStatus::DEATH;
            }
        });

        /////////////////////////////// Take reaction to the enemies with the player ///////////////////////////////
        let cloned_game2 = Arc::clone(&arc_game);

        let handle_enemy_accident_with_player = std::thread::spawn(move || {
            let mut game = cloned_game2.lock().unwrap();
            let (player_i, player_j) = (game.player_i, game.player_j);

            let game_status: bool = game.enemies.iter().any(|enemy| {
                (enemy.location.element_j - 1..enemy.location.element_j + 1).contains(&player_i)
                    && enemy.location.element_i == player_j
            });

            if game_status {
                game.game_staus = GameStatus::DEATH;
            }
        });

        /////////////////////////////// Take reaction to the fuel chars. ///////////////////////////////
        let cloned_game3 = Arc::clone(&arc_game);

        let handle_fuel_reaction = std::thread::spawn(move || {
            let mut game = cloned_game3.lock().unwrap();
            let (player_i, player_j) = (game.player_i, game.player_j);

            let res: bool = game.fuels.iter().any(|fuel| {
                (fuel.location.element_j - 2..fuel.location.element_j + 2).contains(&player_i)
                    && (fuel.location.element_i == player_j)
            });

            if res {
                game.gas += 40;
            }
        });

        /////////////////////////////// Take reaction to the enemies with the player ///////////////////////////////
        let mut enemies_to_remove: Vec<usize> = Vec::new();
        let mut game_in_main_thread = arc_game.lock().unwrap();

        // todo: add rayon parallelization for this nested loops.
        for (idx, enemy) in &mut game_in_main_thread.enemies.iter_mut().enumerate() {
            for bullet in Arc::clone(&arc_game).lock().unwrap().bullets.iter_mut() {
                if bullet.active
                    && (enemy.location.element_i - 2..enemy.location.element_i + 2)
                        .contains(&bullet.location.element_i)
                    && (enemy.location.element_j - 2..enemy.location.element_j + 2)
                        .contains(&bullet.location.element_j)
                {
                    enemy.logo = ' '.to_string();
                    enemies_to_remove.push(idx);
                    sleep(Duration::from_millis(100));
                    bullet.active = false;
                    bullet.logo = ' '.to_string();
                }
            }
        }

        enemies_to_remove.sort_unstable_by(|a, b| b.cmp(a)); // Sort in reverse order
        for idx in enemies_to_remove {
            game_in_main_thread.enemies.remove(idx);
            game_in_main_thread.enemy_killed += 1;
        }

        /////////////////////////////// Take reaction to bottom of the screen ///////////////////////////////
        let cloned_game4 = Arc::clone(&arc_game);

        let handle_bottom_of_the_game = std::thread::spawn(move || {
            let mut game = cloned_game4.lock().unwrap();
            let (player_i, player_j) = (game.player_i, game.player_j);

            let max_screen_j = game.max_screen_j;
            game.enemies
                .retain(|enemy| enemy.location.element_i < max_screen_j - 3);

            game.fuels.retain(|fuel| {
                !((fuel.location.element_j - 1..fuel.location.element_j + 1).contains(&player_i)
                    && (fuel.location.element_i == player_j)
                    || (fuel.location.element_i > max_screen_j - 3))
            });
        });

        handle_accidents.join().unwrap();
        handle_enemy_accident_with_player.join().unwrap();
        handle_fuel_reaction.join().unwrap();
        handle_bottom_of_the_game.join().unwrap();

        Ok(self)
    }
}

pub fn handle_sound(sound_file: String, time_speed: f32) {
    let (_stream, handle) = rodio::OutputStream::try_default().unwrap();
    let sink = rodio::Sink::try_new(&handle).unwrap();

    let file = std::fs::File::open(sound_file).unwrap();
    sink.append(rodio::Decoder::new(BufReader::new(file)).unwrap());

    sink.set_speed(time_speed);
    sink.sleep_until_end();
}
