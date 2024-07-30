use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{poll, read, Event, KeyCode},
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{enable_raw_mode, size, Clear, ClearType},
    ExecutableCommand, QueueableCommand,
};
use inline_colorization::*;
use ndarray::{Array, Array2};
use rand::prelude::*;
use river_raid::*;
use std::cell::RefCell;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::{
    borrow::BorrowMut,
    cell::Ref,
    io::{stdout, Result, Stdout, Write},
    rc::Rc,
    thread::{self, sleep},
    vec,
};
// use shuttle_actix_web::ShuttleActixWeb;


mod server;

fn main2() -> Result<()> {
    let mut screen = stdout();
    enable_raw_mode().unwrap();
    screen.execute(Hide).unwrap();
    screen
        .execute(crossterm::terminal::SetTitle("River Raid Game"))
        .unwrap();

    let mut nd2array = Game2DMatrix::new();

    nd2array.initialize_ground(&mut screen).unwrap();
    let rc_nd2array2 = Rc::new(nd2array.clone());


    while nd2array.clone().game_staus == GameStatus::ALIVE {
        // implementing the keyboard binding.
        if poll(Duration::from_millis(5))? {
            let key = read().unwrap();
            while poll(Duration::from_millis(0)).unwrap() {
                let _ = read();
            }

            if let Event::Key(event) = key { 
                match event.code {
                    KeyCode::Char('q') => {
                        break;
                    }
                    KeyCode::Right => {
                        if nd2array.player_i + 1 < nd2array.max_screen_i {
                            nd2array.player_i += 2;
                        }
                    }
                    KeyCode::Left => {
                        if nd2array.player_i - 1 > 0 {
                            nd2array.player_i -= 2;
                        }
                    }
                    KeyCode::Up => {
                        if nd2array.player_j - 1 > 0 {
                            nd2array.player_j -= 1;
                        }
                    }
                    KeyCode::Down => {
                        if nd2array.player_j + 1 < nd2array.max_screen_j {
                            nd2array.player_j += 1;
                        }
                    }
                    KeyCode::Char(' ') => {
                        thread::spawn(move || {
                            handle_sound(
                                "src/assets/laser_ray_zap_singleshot.wav".to_string(),
                                1.5,
                            );
                        });

                        nd2array.bullets.push(Bullet {
                            location: Location {
                                element_i: nd2array.player_j,
                                element_j: nd2array.player_i,
                            },
                            active: true,
                            logo: '🔥'.to_string(),
                        });
                    }
                    _ => {}
                }
            }
        }

        sleep(Duration::from_millis(60));
        
        nd2array.borrow_mut()
        .draw(
            &mut screen,
            rand::thread_rng().gen_bool(0.1),
            rand::thread_rng().gen_bool(0.01),
        ).unwrap();
    
        nd2array
        .shift_ground_loc(rand::thread_rng().gen_bool(0.5)).unwrap();

        nd2array.reactions().unwrap();
    }

    handle_sound("src/assets/game_over.wav".to_string(), 1.0);

    screen.flush().unwrap();
    screen.execute(Show)?;
    screen.queue(MoveTo(Rc::clone(&rc_nd2array2).max_screen_i / 2, 0))?
        .queue(Print(format!("{color_green}Thanks for playing{color_reset}\n")))?
        .queue(Clear(ClearType::All))?;
    Ok(())
}



// todo this function should applied at a separate cargo
#[actix_web::main]
async fn main() -> Result<()>{
    println!("Running web server...");
    server::run_server().await
}