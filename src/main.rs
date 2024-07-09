use std::{borrow::BorrowMut, io::{stdout, Result, Stdout, Write}, thread::sleep};
use crossterm::{
    cursor::{Hide, MoveTo, Show}, event::{self, poll, read, Event, KeyCode}, execute, style::{
        Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor}, 
        terminal::{enable_raw_mode, size, Clear, ClearType}, ExecutableCommand, QueueableCommand
};
use ndarray::{Array2, ArrayBase, Dim, OwnedArcRepr, Array};
use inline_colorization::*;
use rand::prelude::*;
use chrono::prelude::*;
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
    fn reactions(&mut self, /*screen: &mut Stdout*/) -> Result<&mut Game2DMatrix>;
    // comprehensive explanation
    fn initialize_ground(&mut self, screen: &mut Stdout) -> Result<&mut Self>;
    // comprehensive explanation
    fn shift_ground_loc(&mut self, change: bool) -> Result<&mut Self>;
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
    screen_mid: u16,
    map: Array2<f64>,
    ground: Vec<(u16, u16)>,
    game_staus: GameStatus,
    initialized: bool,
    logo: String,
}

impl GameStructure for Game2DMatrix {
    /// @notice this function will use at the beginning of the game to initialize the ground borders.
    /// @dev this function will use in the draw function is self.initialize was false.
    fn initialize_ground(&mut self, screen: &mut Stdout) -> Result<&mut Self> {
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
        Ok(self)
    }

    fn draw(&mut self, screen: &mut Stdout) -> Result<()> {
        screen.queue(Clear(ClearType::All))?;

        // draw the map as first scence
        for i in 0..(self.map.row(0).len()) {
            screen.queue(MoveTo(0, i as u16))? // (i, j)
                .queue(Print("+".repeat(self.ground[i].0 as usize)))?
                .queue(MoveTo(self.ground[i].1, i as u16))?
                .queue(Print("+".repeat((self.max_screen_i - self.ground[i].1) as usize)))?;
        }

        // draw the player
        screen.queue(MoveTo(self.player_i, self.player_j))?;
        screen.queue(Print(&self.logo))?;
        
        screen.flush()?;            
        Ok(())
    }

    fn shift_ground_loc(&mut self, change: bool) -> Result<&mut Self> {
        for i in (1..self.map.row(0).len()).rev() {
            self.ground[i] = self.ground[i - 1];
        }

        let mut rng = rand::thread_rng();
        let delta = rng.gen_range(1..6);

        // TODO: handle the ground border change more intelligently and less artificially.
        if change && (self.ground[1].1 < self.max_screen_i - 5) {
            // self.ground[0] = ((self.screen_mid - delta), (self.screen_mid + delta));
            self.ground[0] = (self.ground[1].0 + delta, self.ground[1].1 + delta);
            Ok(self)
        } else if self.ground[1].0 > 3 {
            self.ground[0] = (self.ground[1].0 - delta, self.ground[1].1 - delta);
            Ok(self)
        } else {
            self.ground[0] = self.ground[1];
            Ok(self)
        }
    }


    fn reactions(&mut self, /*screen: &mut Stdout*/) -> Result<&mut Self> {
        let user_j: usize = self.player_j as usize;

        // handling the boat accidentation with ground
        if self.player_i <= self.ground[user_j].0 || self.player_i >= self.ground[user_j].1
            // || (self.player_j == self.ground[user_j].0 || self.player_j == self.ground[user_j].)
        {
            self.game_staus = GameStatus::DEATH;
        }

        Ok(self)
    }
}


fn main() -> Result<()> {
    let mut screen = stdout();
    enable_raw_mode().unwrap();
    screen.execute(Hide).unwrap();

    
    // initialize the game information
    let (max_i, max_j) = size().unwrap();

    let mut nd2array = &mut Game2DMatrix {
        player_i: max_i / 2,
        player_j: max_j - 10,
        max_screen_i: max_i, 
        max_screen_j: max_j,
        screen_mid: max_i / 2,
        map: Array::from_shape_vec(
            (max_i as usize, max_j as usize), 
            vec![0.0; (max_i*max_j) as usize]).unwrap(),
        // ground: vec![((max_i / 2) - 10, (max_i / 2) + 10); max_j as usize],
        ground: vec![(0,0); max_j as usize],
        game_staus: GameStatus::ALIVE,
        initialized: false,
        logo: '⛵'.to_string(),
    };

    nd2array = nd2array.initialize_ground(&mut screen)?;
    
    while nd2array.game_staus == GameStatus::ALIVE {
        let now = Utc::now().timestamp();

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

                        KeyCode::Right => if nd2array.player_i + 1 < nd2array.max_screen_i { nd2array.player_i += 1; },

                        KeyCode::Left => if nd2array.player_i - 1 > 0 { nd2array.player_i -= 1; },

                        KeyCode::Up => if nd2array.player_j - 1 > 0 { nd2array.player_j -= 1; },

                        KeyCode::Down => if nd2array.player_j + 1 < nd2array.max_screen_j { nd2array.player_j += 1; },
                        _ => {}
                    }
                },
                _ => {}
            }
        }
        sleep(Duration::from_millis(100));
        nd2array = nd2array.reactions().unwrap();
        nd2array.draw(&mut screen).unwrap();
        nd2array = nd2array.shift_ground_loc(rand::thread_rng().gen_bool(0.5)).unwrap();

        if nd2array.game_staus == GameStatus::DEATH { break; }
    }

    screen.flush().unwrap();
    screen.execute(Show)?;
    screen.queue(MoveTo(nd2array.max_screen_i / 2, 0))?;
    screen.queue(Print(format!("{color_green}Thanks for playing{color_reset}\n")))?;

    Ok(())
}
    


// fn main() {
//     let mut screen = stdout();
//     enable_raw_mode().unwrap();
//     // screen.execute(Hide).unwrap();

    
//     // initialize the game information
//     let (max_i, max_j) = size().unwrap();

//     let mut nd2array = &mut Game2DMatrix {
//         player_i: max_i / 2,
//         player_j: max_j - 10,
//         max_screen_i: max_i, 
//         max_screen_j: max_j,
//         map: Array::from_shape_vec(
//             (max_i as usize, max_j as usize), 
//             vec![0.0; (max_i*max_j) as usize]).unwrap(),
//         // ground: vec![((max_i / 2) - 10, (max_i / 2) + 10); max_j as usize],
//         ground: vec![(0,0); max_j as usize],
//         game_staus: GameStatus::ALIVE,
//         initialized: false
//     };

//     nd2array = nd2array.initialize_ground(&mut screen).unwrap();

//     for i in 0..nd2array.ground.len() {
//         println!("{:?}\n", nd2array.ground[i]);
//     }
// }