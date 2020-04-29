fn main() {
    let word_list = get_word_list();

    let secret_word = get_random_word(&word_list);
    let mut game_state = GameState::new(secret_word);

    loop {
        match game_state.run_loop_iteration() {
            LoopState::Continue => (),
            LoopState::Break => break,
        }
    }
}

fn get_word_list() -> std::path::PathBuf {
    use std::env;
    let mut args = env::args().skip(1);
    let input_file_path = match args.next() {
        Some(path) => path,
        None => "input".into(),
    };
    match args.next() {
        Some(_) => {
            eprintln!("Unknown arguments encountered. Expected at most 1 arg.");
            std::process::exit(1);
        }
        None => (),
    }
    input_file_path.into()
}

fn get_random_word<T: AsRef<std::path::Path>>(path: T) -> String {
    use rand::{seq::IteratorRandom, thread_rng};
    use std::{
        fs::File,
        io::{BufRead, BufReader},
    };
    let file: File = match File::open(path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Cannot read file.");
            eprintln!("{}", e);
            std::process::exit(1);
        }
    };
    let file = BufReader::new(file);
    let mut rng = thread_rng();
    let line = file
        .lines()
        .map(Result::unwrap)
        .filter(|s| s.trim().len() > 0)
        .choose(&mut rng)
        .expect("The Word list is empty");
    line.trim().to_lowercase()
}

#[derive(Debug, Clone)]
struct GameState {
    secret_word: String,
    lives: u8,
    letters_tried: Vec<char>,
    prev_status: Option<UserInputState>,
}

impl GameState {
    fn new(secret_word: String) -> Self {
        Self {
            secret_word,
            lives: 5,
            letters_tried: Vec::new(),
            prev_status: None,
        }
    }

    fn run_loop_iteration(&mut self) -> LoopState {
        Self::title_text();
        Self::print_lives_and_letters(self.lives, &self.letters_tried);
        Self::print_hangman(self.lives);
        Self::print_prev_status(&self.prev_status);
        if self.lives == 0 {
            println!("You Lost!");
            println!("The word was: {}", self.secret_word);
            return LoopState::Break;
        }
        if let LoopState::Break = self.print_masked_word() {
            println!("You Won!");
            return LoopState::Break;
        }
        self.prev_status = Some(self.prompt_and_get_input());
        LoopState::Continue
    }

    fn title_text() {
        println!("HANGMAN: Guess the Word!!");
    }

    fn print_lives_and_letters(lives: u8, letters: &[char]) {
        println!("Lives: {}", lives);
        print!("Tried Letters: ");
        letters.iter().for_each(|c| print!("{} ", c));
        println!();
    }

    fn print_hangman(lives: u8) {
        let figure = match lives {
            0 => {
                " _________
|         |
|         XO
|        /|\\
|        / \\
|
|"
            }

            1 => {
                " _________
|         |
|         O
|        /|\\
|        / \\
|        |||
|        |||"
            }
            2 => {
                " _________
|
|         O
|        /|\\
|        / \\
|        |||
|        |||"
            }

            3 => {
                " _________
|
|
|         O
|        /|\\
|        / \\
|        |||"
            }

            4 => {
                " _________
|
|
|
|         O
|        /|\\
|        / \\"
            }

            5 => {
                "\
|
|
|
|         O
|        /|\\
|        / \\"
            }

            _ => {
                eprintln!("Some error occured. Unknown number of lives.");
                std::process::exit(2)
            }
        };
        println!("{}", figure);
    }

    fn print_prev_status(status: &Option<UserInputState>) {
        use UserInputState::*;
        match status {
            None => println!(),
            Some(RepeatedCharacter) => println!("You have already tried this letter previously."),
            Some(MultipleCharacters) => println!("Enter a single character at a time."),
            Some(CorrectCharacter) => println!("Yay! You got one."),
            Some(WrongCharacter) => println!("Uh Oh! You misguessed."),
        }
    }

    fn print_masked_word(&self) -> LoopState {
        let mut state = LoopState::Break;
        for c in self.secret_word.chars() {
            if self.letters_tried.contains(&c) || c.is_whitespace() {
                print!("{} ", c);
            } else {
                state = LoopState::Continue;
                print!("_ ");
            }
        }
        println!();
        state
    }

    fn prompt_and_get_input(&mut self) -> UserInputState {
        println!("Enter your guess:");
        use std::io;
        let mut buf = String::new();
        loop {
            io::stdin()
                .read_line(&mut buf)
                .expect("Failed to read input");
            let text = buf.trim();
            if text.len() == 0 {
                buf.clear();
            } else {
                return self.process_input(text);
            }
        }
    }

    fn process_input(&mut self, text: &str) -> UserInputState {
        if text.chars().nth(1).is_some() {
            UserInputState::MultipleCharacters
        } else {
            let c = text.chars().nth(0).unwrap();
            if self.letters_tried.contains(&c) {
                UserInputState::RepeatedCharacter
            } else if self.secret_word.contains(c) {
                self.letters_tried.push(c);
                UserInputState::CorrectCharacter
            } else {
                self.lives -= 1;
                UserInputState::WrongCharacter
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum UserInputState {
    MultipleCharacters,
    RepeatedCharacter,
    CorrectCharacter,
    WrongCharacter,
}

enum LoopState {
    Continue,
    Break,
}
