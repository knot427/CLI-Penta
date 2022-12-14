use core::fmt;
use std::{io, thread};

const BOARD_USIZE:usize = 19;
const BOARD_ISIZE:isize = BOARD_USIZE as isize;
const SINGLE_PLAYER:bool = false;
const BLACK:i8 = 1;
const WHITE:i8 = -1;
const EMPTY:i8 = 0;
const DIRECTION_VECTOR: [(isize, isize); 9] = [(-1, -1), (0, -1), (1, -1), (-1, 0), (0, 0), (1, 0), (-1, 1), (0, 1), (1, 1)];
#[derive(Clone)]
struct GameState {
    board: Vec<i8>,
    black_captures:usize,
    white_captures:usize,
    next_player:i8,
    finished:i8,
    top_left:(isize,isize),
    bottom_right:(isize,isize),
}

impl GameState {
    fn new() -> GameState {
        GameState { 
            board: vec![0; BOARD_USIZE*BOARD_USIZE],
            black_captures: 0,
            white_captures: 0,
            next_player: WHITE,
            finished: 0,
            top_left: (255, 255),
            bottom_right: (0, 0),
        }
    }

    fn index_to_coord(i:usize) -> Result<(isize, isize), ()> {
        let i: isize = i.try_into().unwrap();

        if i < BOARD_ISIZE * BOARD_ISIZE {
            Ok((i % BOARD_ISIZE, i / BOARD_ISIZE))
        } else {
            Err(())
        }
    }

    fn coord_to_index((x, y): (isize, isize)) -> Result<usize, ()> {
        if x >= 0 && x < BOARD_ISIZE && y >= 0 && y < BOARD_ISIZE {
            Ok((x + (y * BOARD_ISIZE)).try_into().unwrap())
        } else {
            Err(())
        }
        
    }

    fn make_play(&mut self, coord: (isize, isize)) -> Result<bool, String> {

        if self.finished != 0 {
            return Err("The Game has ended. No more moves can be played.".to_string());
        }

        if !self.place_piece(coord) {
            return Err(format!("Could not place piece at {}, {}.", coord.0, coord.1));
        }

        self.top_left = (self.top_left.0.min((coord.0 - 1).max(0)), self.top_left.1.min((coord.1 - 1).max(0)));
        self.bottom_right = (self.bottom_right.0.max((coord.0 + 1).min(BOARD_ISIZE)), self.bottom_right.1.max((coord.0 + 1).min(BOARD_ISIZE)));

        self.check_captures(coord);

        if self.check_capture_win() || self.check_line(coord) {
            self.finished = self.next_player;
        }

        self.next_player *= -1;

        Ok(self.finished != 0)
    }

    fn place_piece(&mut self, coord: (isize, isize)) -> bool {
        let index = match GameState::coord_to_index(coord) {
            Ok(index) => index,
            Err(_) => return false,
        };

        if self.board[index] != 0 {
            return false;
        } else {
            self.board[index] = self.next_player;
            return true;
        }
    }

    fn get_piece(&self, coord: (isize, isize)) -> Result<i8, ()> {
        let index = match GameState::coord_to_index(coord) {
            Ok(index) => index,
            Err(_) => return Err(()),
        };
        Ok(self.board[index])
    }

    fn get_piece_safe(&self, coord: (isize, isize)) -> i8 {
        let index = match GameState::coord_to_index(coord) {
            Ok(index) => index,
            Err(_) => return 0,
        };
        self.board[index]
    }

    fn check_captures(&mut self, (x, y): (isize, isize)){
        for (xdir, ydir) in DIRECTION_VECTOR.into_iter() {
            if xdir == 0 && ydir == 0 {
                continue;
            }
            let first_piece = self.get_piece((x + (xdir * 1),  y + (ydir * 1))) == Ok(self.next_player * -1);
            let second_piece = self.get_piece((x + (xdir * 2),  y + (ydir * 2))) == Ok(self.next_player * -1);
            let third_piece = self.get_piece((x + (xdir * 3),  y + (ydir * 3))) == Ok(self.next_player);
            if first_piece && second_piece && third_piece {
                self.remove_piece((x + (xdir * 1),  y + (ydir * 1))).unwrap();
                self.remove_piece((x + (xdir * 2),  y + (ydir * 2))).unwrap();
                if self.next_player == WHITE {
                    self.white_captures += 1;
                } else {
                    self.black_captures += 1;
                }
            } 
        }
    }
    
    fn check_line(&self, (x, y): (isize, isize)) -> bool {
        for (xdir, ydir) in DIRECTION_VECTOR.into_iter() {
            if xdir == 0 && ydir == 0 {
                continue;
            }
            let first_piece = self.get_piece((x + (xdir * 1), y + (ydir * 1))) == Ok(self.next_player);
            let second_piece = self.get_piece((x + (xdir * 2), y + (ydir * 2))) == Ok(self.next_player);
            let third_piece = self.get_piece((x + (xdir * 3), y + (ydir * 3))) == Ok(self.next_player);
            let fourth_piece = self.get_piece((x + (xdir * 4), y + (ydir * 4))) == Ok(self.next_player);
            if first_piece && second_piece && third_piece && fourth_piece {
                return true;
            } 
        }
        return false;
    }
    
    fn check_capture_win(&self) -> bool {
        self.black_captures >= 5 || self.white_captures >= 5
    }
    
    fn remove_piece(&mut self, coord: (isize, isize)) -> Result<(), ()> {
        let index = match GameState::coord_to_index(coord) {
            Ok(index) => index,
            Err(_) => return Err(()),
        };
        self.board[index] = 0;
        Ok(())
    }
}

impl fmt::Display for GameState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut out = "\n\n a  b  c  d  e  f  g  h  i  j  k  l  m  n  o  p  q  r  s  \n┌──┬──┬──┬──┬──┬──┬──┬──┬──┬──┬──┬──┬──┬──┬──┬──┬──┬──┬──┐\n".to_owned();
        let middle =    "├──┼──┼──┼──┼──┼──┼──┼──┼──┼──┼──┼──┼──┼──┼──┼──┼──┼──┼──┤\n";
        let bottom =    "└──┴──┴──┴──┴──┴──┴──┴──┴──┴──┴──┴──┴──┴──┴──┴──┴──┴──┴──┘\n";
        for y in 0..BOARD_USIZE {
            out += "│";
            for x in 0..BOARD_USIZE {
                let piece = self.board[y * BOARD_USIZE + x];
                if piece == 0 {
                    out += "  │";
                } else if piece == 1 {
                    out += "[]│";
                } else {
                    out += "██│";
                }
                
            }
            out += &format!("{}\n", y);
            if y == BOARD_USIZE - 1 {
                out += bottom;
            } else {
                out += middle
            }
        }
        out += &format!("White Captures: {}/10      Black Captures: {}/10\n", self.white_captures * 2, self.black_captures * 2);
        write!(f, "{}", out)
    }
}

fn main() {
    let mut game = GameState::new();
    print!("{}", game);
    let mut player_one_turn = true;

    loop {
        let win;
        let play_coord: (isize, isize);
        if player_one_turn || SINGLE_PLAYER {
            play_coord = get_user_placement(&game);
        } else {
            play_coord = automated_opponent(&game, BLACK);
        }

        win = match game.make_play(play_coord){
            Ok(win) => win,
            Err(_) => continue,
        };

        print!("{}", game);

        if win {
            if player_one_turn {
                print!("White wins!");
            } else {
                print!("Black wins!");
            }
            break;
        }
        
        player_one_turn = !player_one_turn;
    }
    
}

fn get_user_placement(game: &GameState) -> (isize, isize) {
    loop {
        let coord = get_user_input();
        if game.get_piece(coord) == Ok(0) {
            return coord;
        } else {
            println!("Invalid piece location.");
        }
    }
}

fn get_user_input() -> (isize, isize) {
    let mut xpos:isize;
    let mut ypos:isize;
    loop {
        let mut user_input = "".to_owned();
        match io::stdin().read_line(&mut user_input) {
            Ok(len) => {
                if len < 2 {
                    println!("invalid input");
                    continue;
                }
            },
            Err(_) => {
                println!("Encountered an error, please try again.");
                continue;
            },
        };

        let mut input_iter = user_input.chars();

        xpos = match input_iter.next().unwrap() {
            'a' => 0,
            'b' => 1,
            'c' => 2,
            'd' => 3,
            'e' => 4,
            'f' => 5,
            'g' => 6,
            'h' => 7,
            'i' => 8,
            'j' => 9,
            'k' => 10,
            'l' => 11,
            'm' => 12,
            'n' => 13,
            'o' => 14,
            'p' => 15,
            'q' => 16,
            'r' => 17,
            's' => 18,
            _ => {
                println!("first character must be a lowercase letter a-s");
                continue;
            },
        };

        let first_n = match input_iter.next().unwrap() {
            '0' => 0,
            '1' => 1,
            '2' => 2,
            '3' => 3,
            '4' => 4,
            '5' => 5,
            '6' => 6,
            '7' => 7,
            '8' => 8,
            '9' => 9,
            _ => {
                println!("Invalid first digit. Must be a number must be between 0 and 18.");
                continue;
            },
        };

        let second_n = match input_iter.next().unwrap() {
            '0' => 0,
            '1' => 1,
            '2' => 2,
            '3' => 3,
            '4' => 4,
            '5' => 5,
            '6' => 6,
            '7' => 7,
            '8' => 8,
            '9' => 9,
            _ => isize::MAX,
        };

        if second_n == isize::MAX {
            ypos = first_n;
        } 
        else {
            ypos = (10 * first_n) + second_n;
        } 
        if ypos >= BOARD_ISIZE {
            println!("Invalid number size: {}. Must be a number must be between 0 and 18.", ypos);
            continue;
        }
        break;
    }
    return (xpos, ypos);
    
}



fn automated_opponent(game: &GameState, colour: i8) -> (isize, isize) {
    min_max_tree_multithread(game, 6, colour)
}

fn min_max_tree_multithread(game: &GameState, depth: isize, colour:i8) -> (isize, isize) {
    
    let mut moves = Vec::<(i128, (isize, isize))>::new();
    let mut thread_handles = Vec::new();
    for (index, space) in game.board.iter().enumerate() {
        let coord = GameState::index_to_coord(index).unwrap();
        if *space == EMPTY && is_relevant(&game, coord) {
            let mut possibility = game.clone();
            possibility.make_play(coord).unwrap();
            thread_handles.push(thread::spawn(move || (alpha_beta(&possibility, depth - 1, i128::MIN, i128::MAX, colour), coord)));
        }
    }

    for thread in thread_handles {
        moves.push(thread.join().unwrap());
    }

    moves.iter().reduce(|acc, i| {
        if acc.0 > i.0 {
            acc
        } else {
            i
        }
    }).unwrap().1
}

fn alpha_beta(game: &GameState, depth: isize, alpha: i128, beta:i128, colour: i8) -> i128 {
    if game.finished != 0 {
        if game.finished == colour {
            return i128::MAX;
        } else {
            return i128::MIN;
        }
    } else if depth <= 0 {
            return min_max_huristic_v2(game, colour);
    }

    if game.next_player == colour {
        let mut value = i128::MIN;
        let mut a = alpha;

        for coord in relevant_coords(game) {
            let mut future = game.clone();
            if future.make_play(coord).is_err() {
                continue;
            }
            value = value.max(alpha_beta(&future, depth - 1, a, beta, colour));
            if value >= beta {
                break;
            }
            a = a.max(value);
        }
        return value;
    } else {
        let mut value = i128::MAX;
        let mut b = beta;
        for coord in relevant_coords(game) {
            let mut future = game.clone();
            if future.make_play(coord).is_err() {
                continue;
            }
            value = value.min(alpha_beta(&future, depth - 1, alpha, b, colour));
            if value <= alpha {
                break;
            }
            b = b.min(value);
        }
        return value;
    }
}

fn relevant_coords(game: &GameState) -> Vec<(isize, isize)> {

    let mut out: Vec<(isize, isize)> = Vec::new();

    for x in game.top_left.0..=game.bottom_right.1 {
        for y in game.top_left.0..=game.bottom_right.1 {
            if game.get_piece((x, y)) == Ok(0) && is_relevant(game, (x, y))  {
                out.push((x, y));
            }
        }
    }

    return out;
}

fn is_relevant(game: &GameState, (x, y): (isize, isize)) -> bool {
    let mut relevant = false;

    for (xdir, ydir) in DIRECTION_VECTOR.into_iter() {
        relevant = relevant || game.get_piece_safe((x + xdir, y + ydir)) != EMPTY;
    }

    relevant
}

fn min_max_huristic_v2(game: &GameState, colour: i8) -> i128 {
    if game.finished != 0 {
        if game.finished == colour {
            return i128::MAX;
        } else {
            return i128::MIN;
        }
    } else {
        let mut score: i128 = 0;
        for (x, y) in relevant_coords(game) {
            for (xdir, ydir) in DIRECTION_VECTOR.into_iter() {
                if xdir == 0 && ydir == 0 {
                    continue;
                }
                let mut count = 0;
                let count_colour = game.get_piece_safe((x + xdir , y + ydir));
                let mut next = game.get_piece_safe((x + (xdir*(count + 1)) , y + (ydir*(count+ 1))));
                if count_colour != EMPTY {
                    while next == count_colour {
                        count += 1;
                        next = game.get_piece_safe((x + (xdir*(count + 1)) , y + (ydir*(count+ 1))));
                    }
                }
                if count_colour != colour {
                    score -= count as i128 * count as i128;
                } else {
                    score += count as i128 * count as i128;
                }
            }
        }
        return score;
    }
    
}