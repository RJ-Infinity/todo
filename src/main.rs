use std::{io::Write, process::exit};
use getch::Getch;
use once_cell::sync::Lazy;
use termsize;

pub enum Colour{
	Default = 0,
	BoldBright = 1,
	NoBoldBright = 22,
	Underline = 4,
	NoUnderline = 24,
	Negative = 7,
	Positive = 27,
	ForegroundBlack = 30,
	ForegroundRed = 31,
	ForegroundGreen = 32,
	ForegroundYellow = 33,
	ForegroundBlue = 34,
	ForegroundMagenta = 35,
	ForegroundCyan = 36,
	ForegroundWhite = 37,
	ForegroundExtended = 38,
	ForegroundDefault = 39,
	BackgroundBlack = 40,
	BackgroundRed = 41,
	BackgroundGreen = 42,
	BackgroundYellow = 43,
	BackgroundBlue = 44,
	BackgroundMagenta = 45,
	BackgroundCyan = 46,
	BackgroundWhite = 47,
	BackgroundExtended = 48,
	BackgroundDefault = 49,
	BrightForegroundBlack = 90,
	BrightForegroundRed = 91,
	BrightForegroundGreen = 92,
	BrightForegroundYellow = 93,
	BrightForegroundBlue = 94,
	BrightForegroundMagenta = 95,
	BrightForegroundCyan = 96,
	BrightForegroundWhite = 97,
	BrightBackgroundBlack = 100,
	BrightBackgroundRed = 101,
	BrightBackgroundGreen = 102,
	BrightBackgroundYellow = 103,
	BrightBackgroundBlue = 104,
	BrightBackgroundMagenta = 105,
	BrightBackgroundCyan = 106,
	BrightBackgroundWhite = 107
}
pub enum ClearType{
	FromCur = 0,
	ToCur = 1,
	All = 2,
}
pub struct IO{}
impl IO{
	fn flush(){ let _ = std::io::stdout().flush(); }
	fn write(text: &str){let _ = std::io::stdout().write_all(text.as_bytes());}
	fn move_cur(x :usize, y:usize){
		Self::write(&format!("\x1b[{}G",x));
		Self::write(&format!("\x1b[{}d",y));
	}
	fn set_up_screen(){
		Self::write("\x1b7");
		Self::write("\x1B[?1049h");
		Self::move_cur(1,1);
	}
	fn restore_screen(){
		Self::write("\x1B[?1049l");
		Self::write("\x1b8");
	}
	fn set_colour(c: Colour){ Self::write(&format!("\x1b[{}m",c as usize)); }
	fn clear_display(c: ClearType){ Self::write(&format!("\x1b[{}J",c as usize)); }
	fn clear_line(c: ClearType){ Self::write(&format!("\x1b[{}K",c as usize)); }
}

enum TodoState{
	Todo,
	Doing,
	Done,
}

struct Todo{
	children: Vec<Todo>,
	name: String,
	description: String,
	state: TodoState,
	open: bool,
}
impl Todo{
	fn new(name:String)->Self{Todo {
		children: Vec::new(),
		name: name,
		description: String::new(),
		state: TodoState::Todo,
		open: false,
	}}
	fn draw(&self, indent: usize, selected: &Todo, width: usize){
		IO::clear_line(ClearType::FromCur);
		if std::ptr::eq(self, selected) {IO::set_colour(Colour::Negative);}
		IO::write(&" ".repeat(indent*2));
		IO::write(if self.open {"▼"} else {"▶"});
		IO::write(" [");
		IO::write(match self.state {
			TodoState::Todo => " ",
			TodoState::Doing => "■",
			TodoState::Done => "x",
		});
		IO::write("] ");
		let loc = 6 + indent*2;
		if self.name.len() > width - loc{
			IO::write(&self.name[..(width - loc - 1)]);
			IO::write("…");
		}else{IO::write(self.name.as_str());}
		IO::write("\n");
		if std::ptr::eq(self, selected) {IO::set_colour(Colour::Default);}
		if self.open{for child in &self.children{ child.draw(indent+1, selected, width); }}
	}
	fn draw_details(&self, start: (usize, usize), width: usize){
		let mut i = 0;
		i+=write_str_with_width(&self.name, start, width);
		IO::move_cur(start.0, start.1+i);
		IO::write(&"=".repeat(width));
		i+=1;
		i+=write_str_with_width(&self.description, (start.0, start.1+i), width);
		IO::move_cur(start.0, start.1+i);
		IO::write(&"=".repeat(width));
	}	
}

#[derive(Debug)]
enum ControlChar{
	Home,
	Up,
	PgUp,
	Left,
	Right,
	End,
	Down,
	PgDown,
	CtrlUp,
	CtrlDown,
	CtrlLeft,
	CtrlRight,
	Unknown(u8),
}
#[derive(Debug)]
enum TermChar{
	Char(char),
	ControlChar(ControlChar),
	Unknown(u8),
}
static GETCH: Lazy<Getch> = Lazy::new(||Getch::new());
fn getch() -> Result<TermChar, std::io::Error>{match GETCH.getch() {Ok(chr) => {match chr{
	chr=> if chr <= 156{
		return Ok(TermChar::Char(chr as char))
	}else if chr == 224{
		match GETCH.getch() {Ok(chr) => {return Ok(TermChar::ControlChar(match chr{
			71=>ControlChar::Home,
			72=>ControlChar::Up,
			73=>ControlChar::PgUp,
			75=>ControlChar::Left,
			77=>ControlChar::Right,
			79=>ControlChar::End,
			80=>ControlChar::Down,
			81=>ControlChar::PgDown,
			141=>ControlChar::CtrlUp,
			145=>ControlChar::CtrlDown,
			115=>ControlChar::CtrlLeft,
			116=>ControlChar::CtrlRight,
			chr=>ControlChar::Unknown(chr),
		}))},Err(e) => { return Err(e); }}
	}else{ return Ok(TermChar::Unknown(chr)); }
}},Err(e) => { return Err(e); }}}

struct Todos{
	data: Vec<Todo>,
	selected: Vec<usize>,
}
macro_rules! get_ptr{($todos: expr/*Vec<Todo>*/, $id: expr/*Vec<usize>*/)=>{{
	let mut curr = &($todos[$id[0]]);
	for id_el in $id.iter().skip(1){curr = &curr.children[*id_el];}
	curr
}}}
macro_rules! get_mut_ptr{($todos: expr/*Vec<Todo>*/, $id: expr/*Vec<usize>*/)=>{{
	let mut curr = &mut ($todos[$id[0]]);
	for id_el in $id.iter().skip(1){curr = &mut curr.children[*id_el];}
	curr
}}}
macro_rules! get_parent_arr{($todos: expr/*Vec<Todo>*/, $id: expr/*Vec<usize>*/)=>{
	if $id.len() > 1 {&get_ptr!($todos, &$id[..$id.len()-1]).children } else {&$todos}
}}
macro_rules! get_mut_parent_arr{($todos: expr/*Vec<Todo>*/, $id: expr/*Vec<usize>*/)=>{
	if $id.len() > 1 { &mut get_mut_ptr!($todos, &$id[..$id.len()-1]).children } else {&mut $todos}
}}
impl Todos{
	fn update_state(&mut self){
		get_mut_ptr!(self.data, self.selected).state = match get_ptr!(self.data, self.selected).state{
			TodoState::Done => TodoState::Todo,
			TodoState::Todo => TodoState::Doing,
			TodoState::Doing => TodoState::Done,
		}
	}
	fn draw(&self, (width, height): (usize,usize)){
		IO::move_cur(1,1);
		let curr = get_ptr!(self.data, self.selected);
		for todo in &self.data{ todo.draw(0, curr, (width/2)-1); }
		IO::clear_display(ClearType::FromCur); // we dont clear the whole screen before redrawing as that causes flicker instead it is done one line at a time
		draw_vertical_line(height, width/2, 1);
		curr.draw_details(((width/2)+1,1), width/2);
		IO::flush();
	}
	fn select_prev(&mut self){if self.selected.last() == Some(&0) {
		if self.selected.len() > 1 { self.selected.pop(); }
	} else {
		*self.selected.last_mut().unwrap() -= 1;
		let curr = get_ptr!(self.data, self.selected);
		if curr.open && curr.children.len() > 0{
			self.selected.push(curr.children.len()-1);
		}
	}}
	fn select_next(&mut self){
		let curr = get_ptr!(self.data, self.selected);
		if curr.open && curr.children.len() > 0 { self.selected.push(0); } else {
			let mut selected_tmp = &self.selected[..];
			while
				selected_tmp.len() > 0 &&
				selected_tmp.last().unwrap()+1 >= get_parent_arr!(
					self.data, selected_tmp
				).len()
			{selected_tmp = &selected_tmp[..selected_tmp.len()-1];}

			if selected_tmp.len() > 0 {
				self.selected.truncate(selected_tmp.len());
				*self.selected.last_mut().unwrap() += 1;
			}
		}
	}
	fn close_sel(&mut self){get_mut_ptr!(self.data, self.selected).open = false;}
	fn open_sel(&mut self){get_mut_ptr!(self.data, self.selected).open = true;}
	fn move_sel_down(&mut self){
		get_mut_ptr!(self.data, self.selected).open = false;
		let index = *self.selected.last().unwrap();
		let next_el = get_parent_arr!(self.data, self.selected).get(index+1);
		if next_el.is_some() {
			if next_el.unwrap().open{
				let item = get_mut_parent_arr!(self.data, self.selected).remove(index);
				get_mut_ptr!(self.data, self.selected).children.insert(0, item);
				self.selected.push(0);
			}else{
				get_mut_parent_arr!(self.data, self.selected).swap(index, index+1);
				*self.selected.last_mut().unwrap() += 1;
			}
		}else if self.selected.len() > 1{
			let item = get_mut_parent_arr!(self.data, self.selected).remove(index);
			self.selected.pop();
			*self.selected.last_mut().unwrap() += 1;
			get_mut_parent_arr!(self.data, self.selected).insert(
				*self.selected.last().unwrap(), item
			);
		}
	}
	fn move_sel_up(&mut self){
		get_mut_ptr!(self.data, self.selected).open = false;
		if self.selected.len() > 1 && self.selected.last() == Some(&0){
			self.selected.pop();
			let item = get_mut_ptr!(self.data, self.selected).children.remove(0);

			get_mut_parent_arr!(self.data, self.selected).insert(
				*self.selected.last().unwrap(), item
			);
		}else if self.selected.last().unwrap() > &0{
			let index = *self.selected.last().unwrap();
			if get_parent_arr!(self.data, self.selected)[index-1].open{
				let parent = get_mut_parent_arr!(self.data, self.selected);
				*self.selected.last_mut().unwrap() -= 1;
				self.selected.push(parent[index-1].children.len());
				let item = parent.remove(index);
				parent[index-1].children.push(item);
			}else{
				get_mut_parent_arr!(self.data, self.selected).swap(index, index-1);
				*self.selected.last_mut().unwrap() -= 1;
			}
		}
	}
}

fn draw_vertical_line(height: usize, col: usize, row: usize){ for i in row..row+height{
	IO::move_cur(col, i);
	IO::write("|");
}}
fn write_str_with_width(text: &String, start: (usize, usize), width: usize)->usize{
	let mut i = 0;
	let mut tmp_text = &text[..];
	while tmp_text.len() > width{
		IO::move_cur(start.0, start.1+i);
		IO::write(&tmp_text[..width]);
		tmp_text = &tmp_text[width..];
		i+=1;
	}
	IO::move_cur(start.0, start.1+i);
	IO::write(&tmp_text);
	return i+1;
}

fn todo_loop(mut todos: Todos){
	IO::set_up_screen();
	if todos.data.len() == 0{todos.data.push(Todo::new("Add Todos".to_string()).into())}
	let mut size;
	loop{
		size = termsize::get().unwrap();
		todos.draw((size.cols as usize, size.rows as usize));
		if size.cols < 10{
			eprintln!("Error. screen not wide enough.");
			break;
		}
		match getch() { Ok(chr) => {match chr{
			TermChar::Char(chr) => match chr{
				'\x03'=>break, // this is ctrl-c
				'\r' | ' '=>todos.update_state(),
				'\t'=>todo!(),
				'\x1b'=>break, // this is esc
				_=>{},// do nothing
			},
			TermChar::ControlChar(chr) => match chr{
				ControlChar::Up=>todos.select_prev(),
				ControlChar::Down=>todos.select_next(),
				ControlChar::Left=>todos.close_sel(),
				ControlChar::Right=>todos.open_sel(),
				ControlChar::CtrlDown=>todos.move_sel_down(),
				ControlChar::CtrlUp=>todos.move_sel_up(),
				_=>{},// do nothing
			},
			_=>{}, // do nothing
		}},Err(e) => {
			eprintln!("{}",e);
			break;
		}};
	}
	IO::restore_screen();
}

fn main() {todo_loop(Todos{data:vec!(Todo{
	children: vec!(
		Todo::new("child1".to_string()),
		Todo::new("child2".to_string()),
	),
	name: "parent".to_string(),
	description: String::new(),
	state: TodoState::Doing,
	open: true,
},Todo::new("parent2".to_string()),), selected:vec!(0)});}
