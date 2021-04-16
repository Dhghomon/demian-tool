use std::collections::VecDeque;
use std::{fs, mem};
use std::io::{Write, BufWriter};

use ansi_term::Colour::{Cyan, Red};
use crossterm::{event::*, queue, terminal::*};

#[derive(Debug)]
struct Content {
    english: VecDeque<VecDeque<String>>,
    german: VecDeque<VecDeque<String>>,

    current_work: (VecDeque<String>, VecDeque<String>), // The current lines
    ready_to_insert: (Vec<String>, Vec<String>),        // Selected words
    data_for_undo: (Vec<String>, Vec<String>),
    content_for_file: Vec<String>, // content saved when pressing enter

    language: Language,
}

impl Content {
    fn display(&self) {
        let english: String = self
            .ready_to_insert
            .0
            .iter()
            .cloned()
            .map(|mut word| {
                word.push(' ');
                word
            })
            .collect();

        let german: String = self
            .ready_to_insert
            .1
            .iter()
            .cloned()
            .map(|mut word| {
                word.push(' ');
                word
            })
            .collect();

        // Makes ***** red to show which language is selected
        match self.language {
            Language::German => {
                println!(
                    " {}{}{}",
                    Cyan.paint(&"*".repeat(18)),
                    Red.paint(&"*".repeat(8)),
                    Cyan.paint(&"*".repeat(32))
                );
            }
            Language::English => {
                println!(
                    " {}{}{}",
                    Cyan.paint(&"*".repeat(32)),
                    Red.paint(&"*".repeat(8)),
                    Cyan.paint(&"*".repeat(18))
                );
            }
        }
        // Now the words we're ready to choose
        println!(" {:>27}\t{}", german, english);

        println!(" {}", Cyan.paint(&"·".repeat(58)));

        let nothing = String::from("---");

        for i in 0..10 {
            println!(
                " {}{:>25}\t{:<26}{:>}",
                Cyan.paint("*"),
                self.current_work.1.get(i).unwrap_or(&nothing),
                self.current_work.0.get(i).unwrap_or(&nothing),
                Cyan.paint("*")
            );
        }
        println!(" {}", Cyan.paint(&"*".repeat(58)));
    }

    fn up(&mut self) {
        match self.language {
            Language::English => {
                if !self.current_work.0.is_empty() {
                    self.ready_to_insert
                        .0
                        .push(self.current_work.0.pop_front().unwrap());
                }
            }
            Language::German => {
                if !self.current_work.1.is_empty() {
                    self.ready_to_insert
                        .1
                        .push(self.current_work.1.pop_front().unwrap());
                }
            }
        }
    }

    fn initiate(&mut self) {
        self.current_work.0 = self.english.pop_front().unwrap();
        self.current_work.1 = self.german.pop_front().unwrap();
    }

    fn down(&mut self) {
        match self.language {
            Language::English => {
                if !self.ready_to_insert.0.is_empty() {
                    self.current_work
                        .0
                        .push_front(self.ready_to_insert.0.pop().unwrap());
                }
            }
            Language::German => {
                if !self.ready_to_insert.1.is_empty() {
                    self.current_work
                        .1
                        .push_front(self.ready_to_insert.1.pop().unwrap());
                }
            }
        }
    }

    fn try_next_line(&mut self) {
        // get rid of first index if there's no German and no English left to work on
        if self.current_work.0.is_empty() && self.current_work.1.is_empty() {
            self.current_work.0.pop_front();
            self.current_work.1.pop_front();

            // bring in the next ones
            self.current_work.0 = self.english.pop_front().unwrap_or(VecDeque::new()); // could just unwrap since it would only panic if the whole book is done
            self.current_work.1 = self.german.pop_front().unwrap_or(VecDeque::new());
            if self.ready_to_insert.0.is_empty() && self.ready_to_insert.1.is_empty() {
                self.ready_to_insert
                    .0
                    .push(self.current_work.0.pop_front().unwrap_or("".to_string())); //get.unwrap.clone
                self.ready_to_insert
                    .1
                    .push(self.current_work.1.pop_front().unwrap_or("".to_string()));
            }
        }
    }
}

#[derive(Debug, PartialEq)]
enum Language {
    English,
    German,
}

fn main() -> crossterm::Result<()> {
    use KeyCode::*;

    let demian = include_str!("Demiantext.txt");

    let mut content = Content {
        english: VecDeque::new(),
        german: VecDeque::new(),
        current_work: (VecDeque::new(), VecDeque::new()),
        ready_to_insert: (vec![], vec![]),
        data_for_undo: (vec![], vec![]),
        language: Language::German,
        content_for_file: vec![],
    };

    let mut demian_file = fs::OpenOptions::new()
        .append(true)
        .open("demian_file.txt")?;

    let mut german = true;
    for line in demian.lines() {
        match german {
            true => {
                content.german.push_back(
                    line.split_whitespace()
                        .map(|word| word.to_string())
                        .collect::<VecDeque<String>>(),
                );
                german = false;
            }
            false => {
                content.english.push_back(
                    line.split_whitespace()
                        .map(|word| word.to_string())
                        .collect::<VecDeque<String>>(),
                );
                german = true;
            }
        }
    }

    content.initiate();

    // declare stdout, first clearing of the screen
    let mut stdout = std::io::stdout();
    queue!(stdout, Clear(ClearType::All))?;
    stdout.flush()?;

    loop {
        content.display();
        // `read()` blocks until an `Event` is available
        match read()? {
            Event::Key(event) => match event.code {
                Left => content.language = Language::German,
                Right => content.language = Language::English,
                Up => content.up(),
                Down => content.down(),
                #[rustfmt::skip]
                Enter => {
                    let english = &content.ready_to_insert.0
                        .iter()
                        .cloned()
                        .map(|mut word| {
                            word.push(' ');
                            word
                        })
                        .collect::<String>();
                    let german = content.ready_to_insert.1
                        .iter()
                        .cloned()
                        .map(|mut word| {
                            word.push(' ');
                            word
                        })
                        .collect::<String>();
                    if english.is_empty() && german.is_empty()
                    {
                        content.content_for_file.push("\n".to_string());
                    } else {
                        content
                            .content_for_file
                            .push(format!("{}|{}\n", german, english));
                    }
                    content.data_for_undo.0 = mem::take(&mut content.ready_to_insert.0);
                    content.data_for_undo.1 = mem::take(&mut content.ready_to_insert.1);

                    // Moving the top words into ready_to_insert
                    content.language = Language::English;
                    content.up();
                    content.language = Language::German;
                    content.up();
                }
                #[rustfmt::skip]
                Delete => { // Delete = undo

                    // First put the word in ready_to_insert back inside current_work
                    while let Some(word) = content.ready_to_insert.0.pop() {
                        content.current_work.0.push_front(word);
                    }
                    while let Some(word) = content.ready_to_insert.1.pop() {
                        content.current_work.1.push_front(word);
                    }

                    // Then the saved data
                    while let Some(word) = content.data_for_undo.0.pop() {
                        content.current_work.0.push_front(word);
                    }
                    while let Some(word) = content.data_for_undo.1.pop() {
                        content.current_work.1.push_front(word);
                    }
                    
                    // Then remove the (deemed by the user as incorrect) data ready to be saved to the file
                    content.content_for_file.pop();
                    
                }
                Esc => {
                    // First add the new content to the file completed so far
                    for line in content.content_for_file {
                        demian_file.write(line.as_bytes())?;
                    }

                    // Then make a new file to store all the remainder
                    // to be loaded next time
                    let new_file = fs::File::create("Demiantext.txt")?;
                    
                    // Put the current words we're working on back into the main content
                    while !content.ready_to_insert.0.is_empty() {
                        content.current_work.0.push_front(content.ready_to_insert.0.remove(0))
                    }
                    while !content.ready_to_insert.1.is_empty() {
                        content.current_work.1.push_front(content.ready_to_insert.1.remove(0))
                    }

                    content.english.push_front(content.current_work.0);
                    content.german.push_front(content.current_work.1);

                    // Lots of tiny write calls so bring in BufWriter
                    let mut writer = BufWriter::new(new_file);

                    for (german, english) in content.german.iter().zip(content.english.iter()) {
                        for word in german {
                            writer.write(word.as_bytes())?;
                            writer.write(" ".as_bytes())?;
                        }
                        writer.write("\n".as_bytes())?;
                        for word in english {
                            writer.write(word.as_bytes())?;
                            writer.write(" ".as_bytes())?;
                        }
                        writer.write("\n".as_bytes())?;
                    }
                    writer.flush()?;

                    break;
                }
                Char('1') => {
                    content.language = Language::German;
                    content.up();
                }
                Char('2') => {
                    content.language = Language::English;
                    content.up();
                }
                _ => {}
            },
            _ => {}
        }
        queue!(stdout, Clear(ClearType::All))?;
        stdout.flush()?;
        content.try_next_line();
    }
    Ok(())
}
