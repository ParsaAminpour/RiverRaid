use std::{borrow::BorrowMut, io::{stdout, Result, Stdout, Write}, thread::sleep};
use crossterm::{
    cursor::{Hide, MoveTo, Show}, event::{self, poll, read, Event, KeyCode}, execute, style::{
        Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor}, 
        terminal::{enable_raw_mode, size, Clear, ClearType}, ExecutableCommand, QueueableCommand
};
use ndarray::{Array2, ArrayBase, Dim, OwnedArcRepr, Array};
use inline_colorization::*;
use rand::prelude::*;
use std::time::Duration;

/*
** GAME PHASES
* keyboard: binding the keyboard to listening to the keys that inserted
* physics: performing the physical changes for the boats or the enemies
* drawing: performing the changes related to the screen.
*/

trait GameStructure {
    // comprehensive explanation
    fn draw(&mut self, screen: &mut Stdout) -> Result<()>;
    // comprehensive explanation
    fn reactions(&mut self, screen: &mut Stdout) -> Result<&mut Game2DMatrix>;
    // comprehensive explanation
    fn initialize_ground(&mut self, screen: &mut Stdout) -> Result<&mut Self>;
}


#[derive(Debug, PartialEq, Eq)]
enum GameStatus {
    ALIVE, DEATH, PAUSED, FUEL_ENDED
}

#[derive(Debug)]
pub struct Game2DMatrix {
    player_i: u16,
    player_j: u16,
    max_screen_i: u16,
    max_screen_j: u16,
    map: Array2<f64>,
    ground: Vec<(u16, u16)>,
    game_staus: GameStatus,
    initialized: bool,
}


impl GameStructure for Game2DMatrix {
    fn draw(&mut self, screen: &mut Stdout) -> Result<()> {
        // draw the map as first scence
        for i in 0..self.map.row(0).len() {
            screen.queue(MoveTo(1, i as u16))?; // (i, j)
            screen.queue(Print("+".repeat(self.ground[i].0 as usize)))?;

            screen.queue(MoveTo(self.ground[i].1, i as u16))?;
            screen.queue(Print("+".repeat((self.max_screen_i - self.ground[i].1) as usize)))?;

            if i > 1 { self.ground[i-1] = self.ground[i]; }
        }

        // draw the player
        screen.queue(MoveTo(self.player_i, self.player_j))?;
        screen.queue(Print("p"))?;
        
        // @audit-info invalid flushing
        screen.flush()?;            
        Ok(())
    }


    fn reactions(&mut self, screen: &mut Stdout) -> Result<&mut Self> {
        let user_j: usize = self.player_j as usize;

        // handling the boat accidentation with ground
        if self.player_i == self.ground[user_j].0 || self.player_i == self.ground[user_j].1 {
            self.game_staus = GameStatus::DEATH;
        }

        Ok(self)
    }

    fn initialize_ground(&mut self, screen: &mut Stdout) -> Result<&mut Self> {
        // initial phase of screen
        screen.queue(Clear(ClearType::All))?;

        for i in 0..self.map.row(0).len() {
            let mut rng = rand::thread_rng();

            if i == 0 {
                let low = self.player_i - 30;
                let high = self.player_i + 30;
                let rand_loc: u16 = rng.gen_range(low..high);
                (self.ground[i].0, self.ground[i].1) = (low, high);
            }
            
            screen.queue(MoveTo(1, i as u16))?; // (i, j)
            screen.queue(Print("+".repeat(rand_loc as usize)))?;

            screen.queue(MoveTo(self.ground[i].1, i as u16))?;
            screen.queue(Print("+".repeat((self.max_screen_i - rand_loc) as usize)))?;
        }

        self.initialized = true;
        Ok(self)
    }

    // fn change_the_ground_border(&mut self, screen: &mut Stdout) -> Result<&mut Self>{
    //     let user_j: usize = self.player_j as usize;

    //     // changing the border of the ground
    //     let mut rand = rand::thread_rng();
    //     let rand_number: u16 = rand.gen();

    //     if rand_number < self.ground[user_j].0 {
    //         self.ground[1].0 -= 1 ;
    //         self.ground[1].1 -= 1;
    //     } else if rand_number > self.ground[1].1 {
    //         self.ground[1].0 += 1;
    //         self.ground[1].1 += 1;
    //     }
    //     self.ground[2] = self.ground[1];
    //     self.ground[0] = (self.ground[1].0, self.ground[1].1);

    //     Ok(self)
    // }
}


fn main() -> Result<()> {
    let mut screen = stdout();
    enable_raw_mode().unwrap();
    screen.execute(Hide).unwrap();

    
    // initialize the game information
    let (max_i, max_j) = size().unwrap();

    let mut nd2array = &mut Game2DMatrix {
        player_i: max_i / 2,
        player_j: max_j / 2,
        max_screen_i: max_i, 
        max_screen_j: max_j,
        map: Array::from_shape_vec(
            (max_i as usize, max_j as usize), 
            vec![0.0; (max_i*max_j) as usize]).unwrap(),
        ground: vec![((max_i / 2) - 10, (max_i / 2) + 10); max_j as usize],
        game_staus: GameStatus::ALIVE,
        initialized: false
    };

    nd2array = nd2array.initialize_ground(&mut screen).unwrap();

    while nd2array.game_staus == GameStatus::ALIVE {
        // implementing the keyboard binding.
        if poll(Duration::from_millis(10))? {
            let key: Event = read().unwrap();
            match key {
                Event::Key(event) => {
                    match event.code {
                        KeyCode::Char('q') => { break; },

                        KeyCode::Right => {
                            if nd2array.player_i + 1 < nd2array.max_screen_i {
                                nd2array.player_i += 1;
                            }
                        }
                        KeyCode::Left => {
                            if nd2array.player_i - 1 > 0 {
                                nd2array.player_i -= 1;
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

                        _ => {}
                    }
                },
                _ => {}
            }
        }

        sleep(Duration::from_millis(100));
        nd2array = nd2array.reactions(&mut screen).unwrap();
        // nd2array = nd2array.change_the_ground_border(&mut screen).unwrap();
        nd2array.draw(&mut screen).unwrap();

        if nd2array.game_staus == GameStatus::DEATH { break; }
    }

    screen.execute(Show)?;
    screen.flush().unwrap();
    screen.queue(MoveTo(nd2array.max_screen_i / 2, 0))?;
    screen.queue(Print(format!("{color_green}Thanks for playing{color_reset}\n")))?;

    Ok(())
}
    

