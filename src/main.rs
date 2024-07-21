use std::{io::{stdout, Result, Stdout, Write}, thread::sleep, vec};
use crossterm::{
    cursor::{Hide, MoveTo, Show}, event::{poll, read, Event, KeyCode}, style::{
        Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor}, terminal::{enable_raw_mode, size, Clear, ClearType}, ExecutableCommand, QueueableCommand
};
use ndarray::{Array2, Array};
use inline_colorization::*;
use rand::prelude::*;
use std::time::Duration;
// use std::thread;
// use std::sync::{Mutex, Arc};

/*
** GAME PHASES
* keyboard: binding the keyboard to listening to the keys that inserted
* physics: performing the physical changes for the boats or the enemies
* drawing: performing the changes related to the screen.
*/
#[derive(Debug, PartialEq, Eq)]
enum GameStatus {
    ALIVE, DEATH, /*PAUSED, FUEL_ENDED*/
}

#[derive(Debug)]
struct Location {
    element_i: u16,
    element_j: u16
}
#[derive(Debug)]
struct Enemy {
    location: Location,
    logo: String,
}
#[derive(Debug)]
struct Bullet {
    location: Location,
    active: bool,
    logo: String,
}

#[derive(Debug)]
struct Fuel {
    location: Location,
    logo: String,
}

#[derive(Debug)]
pub struct Game2DMatrix {
    player_i: u16,
    player_j: u16,
    max_screen_i: u16,
    max_screen_j: u16,
    screen_mid: u16,
    map: Array2<f64>,
    ground: Vec<(u16, u16)>,
    enemies: Vec<Enemy>,
    bullets: Vec<Bullet>,
    fuels: Vec<Fuel>,
    game_staus: GameStatus,
    score: u32,
    gas: u32,
    enemy_killed: u32,
    initialized: bool,
    logo: String,
}

impl Game2DMatrix {
    fn new() -> Game2DMatrix {
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
                vec![0.0; (max_i*max_j) as usize]).unwrap(),
            ground: vec![(0,0); max_j as usize],
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
    fn initialize_ground(&mut self, screen: &mut Stdout) -> Result<()> {
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


    fn draw(&mut self, screen: &mut Stdout,  show_enemy: bool, show_fuel: bool) -> Result<()> {
        screen.queue(Clear(ClearType::All))?;

        // draw the map as first scence
        for j in 0..(self.map.row(0).len()) {
            screen.queue(MoveTo(0, j as u16))? // (i, j)
                .queue(SetForegroundColor(Color::Green))?
                .queue(SetBackgroundColor(Color::Green))?
                .queue(Print(" ".repeat(self.ground[j].0 as usize)))?

                .queue(MoveTo(self.ground[j].0, j as u16))?
                .queue(SetBackgroundColor(Color::Blue))?
                .queue(Print(" ".repeat((self.ground[j].1 - self.ground[j].0) as usize)))?

                .queue(MoveTo(self.ground[j].1, j as u16))?
                .queue(SetBackgroundColor(Color::Green))?
                .queue(Print(" ".repeat((self.max_screen_i - self.ground[j].1) as usize)))?
                .queue(ResetColor)?;
        }

        for bullet in self.bullets.iter() {
            screen.queue(MoveTo(bullet.location.element_j, bullet.location.element_i))?
                .queue(SetBackgroundColor(Color::Blue))?
                .queue(Print(&bullet.logo))?;
        }

        // adjust furl in the posibility of 10% of situations.
        if show_fuel {
            self.fuels.push( Fuel {
                location: Location {
                    element_i: 2,
                    element_j: rand::thread_rng().gen_range(self.ground[2].0..self.ground[2].1)
                },
                logo: '⛽'.to_string()
            })
        }

        for fuel in self.fuels.iter() {
            screen.queue(MoveTo(fuel.location.element_j, fuel.location.element_i))?
                .queue(SetBackgroundColor(Color::Blue))?
                .queue(Print(&fuel.logo))?;
        }

        // adjust enemy in the posibility of 10% of situations
        if show_enemy {
            self.enemies.push( Enemy {
                location: Location { 
                    element_i: 2, 
                    element_j: rand::thread_rng().gen_range(self.ground[2].0..self.ground[2].1)
                },
                logo: '👾'.to_string(),
            })
        }

        for enemy in self.enemies.iter() {
            screen.queue(MoveTo(enemy.location.element_j, enemy.location.element_i))?
                .queue(SetBackgroundColor(Color::Blue))?
                .queue(Print(&enemy.logo))?
                .queue(ResetColor)?;
        }

        // draw the player
        screen.queue(MoveTo(self.player_i, self.player_j))?
            .queue(SetBackgroundColor(Color::Blue))?
            .queue(Print(&self.logo))?;

        // draw the game scores and status
        let scores_position = (self.max_screen_i / 13, self.max_screen_j / 13);

        screen.queue(SetBackgroundColor(Color::DarkGrey))?
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
        Ok(())
    }


    /// @notice this function perform the elements' movement during the game loop i.e. bullets, enemies, etc.
    /// @dev this function will be called after the draw function to get the modified nd2array game information.
    fn shift_ground_loc(&mut self, change: bool) -> Result<()> {
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
        if self.score % 2 == 0 { self.gas -= 1; }

        if change && (self.ground[1].1 < self.max_screen_i - 5) {
            self.ground[0] = (self.ground[1].0 + delta, self.ground[1].1 + delta);
        } else if self.ground[1].0 > delta {
            self.ground[0] = (self.ground[1].0 - delta, self.ground[1].1 - delta);
        } else {
            self.ground[0] = self.ground[1];
        }
        Ok(())
    }


    fn reactions(&mut self, /*screen: &mut Stdout*/) -> Result<()> {
        let user_j: usize = self.player_j as usize;

        if self.gas == 0 {
            self.game_staus = GameStatus::DEATH;
        }

        // handling the boat accidentation with ground
        if self.player_i <= self.ground[user_j].0 || self.player_i >= self.ground[user_j].1
        {
            self.game_staus = GameStatus::DEATH;
        }

        // enemies
        let mut enemies_to_remove:Vec<usize> = vec![];

        for (idx, enemy) in self.enemies.iter_mut().enumerate() {
            // player collision with the enemies in the ground.
            if (enemy.location.element_j-1..enemy.location.element_j+1).contains(&self.player_i) && 
                enemy.location.element_i == self.player_j {
                self.game_staus = GameStatus::DEATH;
            }

            // the reaction related to the player's bullets verses the enemies.
            for bullet in self.bullets.iter_mut() {
                if bullet.active && (enemy.location.element_i-1..enemy.location.element_i+1).contains(&bullet.location.element_i) &&
                    (enemy.location.element_j-2..enemy.location.element_j+2).contains(&bullet.location.element_j) 
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
            self.enemies.remove(idx);
            self.enemy_killed += 1;
        }

        // Take reaction to the fuel chars.
        for fuel in self.fuels.iter() {
            if (fuel.location.element_j-2..fuel.location.element_j+2).contains(&self.player_i) &&
            (fuel.location.element_i == self.player_j) { self.gas += 30; }
        }
        

        self.enemies.retain(|enemy| {
            enemy.location.element_i < self.max_screen_j - 3
        });

        self.fuels.retain(|fuel|
            !((fuel.location.element_j-1..fuel.location.element_j+1).contains(&self.player_i) &&
                (fuel.location.element_i == self.player_j) ||
                (fuel.location.element_i > self.max_screen_j - 3)
            )
        );

        Ok(())
    }
}


fn main() -> Result<()> {
    let mut screen = stdout();
    enable_raw_mode().unwrap();
    screen.execute(Hide).unwrap();

    let mut nd2array = Game2DMatrix::new();

    nd2array.initialize_ground(&mut screen).unwrap();
    
    while nd2array.game_staus == GameStatus::ALIVE {
        // implementing the keyboard binding.
        if poll(Duration::from_millis(10))? {

            let key = read().unwrap();
            while poll(Duration::from_millis(0)).unwrap() {
                let _ = read();
            }
            match key {
                Event::Key(event) => {
                    match event.code {
                        KeyCode::Char('q') => { break; },

                        KeyCode::Right => if nd2array.player_i + 1 < nd2array.max_screen_i { nd2array.player_i += 2; },

                        KeyCode::Left => if nd2array.player_i - 1 > 0 { nd2array.player_i -= 2; },

                        KeyCode::Up => if nd2array.player_j - 1 > 0 { nd2array.player_j -= 1; },

                        KeyCode::Down => if nd2array.player_j + 1 < nd2array.max_screen_j { nd2array.player_j += 1; },

                        KeyCode::Char(' ') => {
                            nd2array.bullets.push(Bullet {
                                location: Location {
                                    element_i: nd2array.player_j,
                                    element_j: nd2array.player_i,
                                },
                                active: true,
                                logo: '🔥'.to_string()
                            });
                        }
                        _ => {}
                    }
                },
                _ => {}
            }
        }
        sleep(Duration::from_millis(100));
        nd2array.reactions().unwrap();

        nd2array.draw(&mut screen, rand::thread_rng().gen_bool(0.1), rand::thread_rng().gen_bool(0.01)).unwrap();

        nd2array.shift_ground_loc(rand::thread_rng().gen_bool(0.5)).unwrap();

        if nd2array.game_staus == GameStatus::DEATH { break; }
    }

    screen.flush().unwrap();
    screen.execute(Show)?;
    screen.queue(MoveTo(nd2array.max_screen_i / 2, 0))?
        .queue(Print(format!("{color_green}Thanks for playing{color_reset}\n")))?;

    Ok(())
}



    // // let's using multi-thread in advance.
    // fn reactions2(&'static mut self, /*screen: &mut Stdout*/) -> Result<&'static mut Self> {
    //     let user_j: usize = self.player_j as usize;

    //     // Player gas check and ground collision.
    //     if (self.gas == 0) || 
    //         (self.player_i <= self.ground[user_j].0 || self.player_i >= self.ground[user_j].1) 
    //     {
    //         self.game_staus = GameStatus::DEATH;
    //     }
        
    //     let game_state: Arc<Mutex<&'static mut Game2DMatrix>> = Arc::new(Mutex::new(self));
    //     let game_state_cloned: Arc<Mutex<&mut Game2DMatrix>> = Arc::clone(&game_state);

    //     // player collision to the ground checks.
    //     let handling_enemy_and_player_collision = thread::spawn(move|| {
    //         let mut locked_game_state = game_state_cloned.lock().unwrap();

    //         let mut death: bool = false;
    //         for enemy in locked_game_state.enemies.iter() {
    //             // let locked_game_state_instance = locked_game_state;
    //             if enemy.location.element_j == locked_game_state.player_i && enemy.location.element_i == locked_game_state.player_j {
    //                 death = true;
    //             }
    //         }
    //         if death { locked_game_state.game_staus = GameStatus::DEATH; };
    //     });


    //     // the reactions related to the enemies.
    //     let handling_enemy_and_bullet_collisions = thread::spawn(move || {
    //         let mut locked_game_state = game_state.lock().unwrap();
            
    //         for bullet in self.bullets.iter_mut() {
    //             if bullet.active && (enemy.location.element_i-3..enemy.location.element_i+3).contains(&bullet.location.element_i) &&
    //                 (enemy.location.element_j-2..enemy.location.element_j+2).contains(&bullet.location.element_j) 
    //             {                
    //                 enemy.logo = ' '.to_string();
    //                 enemies_to_remove.push(idx);
    //                 sleep(Duration::from_millis(100));
    //                 bullet.active = false;
    //                 bullet.logo = ' '.to_string();
    //             }
    //         }
    //     });
        
    //     // the reaction related to the fuels.

    //     // remove the unused ground chracters at the botton of the ground.
        
    //     // handle the unfullfiled threads.
        
    //     handling_enemy_and_player_collision.join().unwrap();
    //     Ok(self)
    // }
