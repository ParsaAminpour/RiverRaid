use std::{borrow::BorrowMut, io::{stdout, Result, Stdout, Write}, rc::Rc, thread::{self, sleep}, vec};
use crossterm::{
    cursor::{Hide, MoveTo, Show}, event::{poll, read, Event, KeyCode}, style::{
        Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor}, terminal::{enable_raw_mode, size, Clear, ClearType}, ExecutableCommand, QueueableCommand
};
use ndarray::{Array2, Array};
use inline_colorization::*;
use rand::prelude::*;
use std::time::Duration;
use river_raid::*;
use std::sync::{Arc, Mutex};
use std::cell::RefCell;

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

            // Bug related to the Refrence counted while single thread and multi-thread processes are combined with eachother.
            let rc_nd2array = Rc::new(RefCell::new(nd2array));

            match key {
                Event::Key(event) => {

                    match event.code {
                        KeyCode::Char('q') => { break; },

                        KeyCode::Right => if nd2array.player_i + 1 < nd2array.max_screen_i { cloned_nd2array.player_i += 2; },

                        KeyCode::Left => if nd2array.player_i - 1 > 0 { nd2array.player_i -= 2; },

                        KeyCode::Up => if nd2array.player_j - 1 > 0 { nd2array.player_j -= 1; },

                        KeyCode::Down => if nd2array.player_j + 1 < nd2array.max_screen_j { nd2array.player_j += 1; },

                        KeyCode::Char(' ') => {
                            let atomic_nd2array = Arc::new(Mutex::new(nd2array));
                            let cloned_atomic_nd2array = Arc::clone(&atomic_nd2array);

                            let handle_bullet_drowing = std::thread::spawn(move || {
                                let mut locked_atomic_nd2array = cloned_atomic_nd2array.lock().unwrap();

                                locked_atomic_nd2array.bullets.push(Bullet {
                                    location: Location {
                                        element_i: Arc::clone(&atomic_nd2array).lock().unwrap().player_j,
                                        element_j: Arc::clone(&atomic_nd2array).lock().unwrap().player_i,
                                    },
                                    active: true,
                                    logo: '🔥'.to_string()
                                });

                            });
                            
                            let handle_sound = thread::spawn(move || {
                                handle_sound2("src/assets/laser_ray_zap_singleshot.wav".to_string());
                            });

                            handle_bullet_drowing.join().unwrap();
                            handle_sound.join().unwrap();
                        }
                        _ => {}
                    }
                },
                _ => {}
            }
        }
        // let rc_nd2array = Rc::new(&nd2array);

        sleep(Duration::from_millis(66));
        nd2array.reactions().unwrap();
        
        nd2array.draw(&mut screen, rand::thread_rng().gen_bool(0.1), rand::thread_rng().gen_bool(0.01)).unwrap();
        
        nd2array.shift_ground_loc(rand::thread_rng().gen_bool(0.5)).unwrap();
        
        // if nd2array.game_staus == GameStatus::DEATH { break; }
    }

    let rc_nd2array2 = Rc::new(nd2array);
    screen.flush().unwrap();
    screen.execute(Show)?;
    screen.queue(MoveTo(Rc::clone(&rc_nd2array2).max_screen_i / 2, 0))?
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
