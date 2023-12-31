use std::{io::Write, panic, cell::RefCell};
use getch::Getch;
use once_cell::sync::Lazy;
use termsize;

#[derive(Clone, Copy)]
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
		panic::set_hook(Box::new(|_| {
			Self::restore_screen();
			Self::flush();
		}));
	}
	fn restore_screen(){
		Self::write("\x1B[?1049l");
		Self::write("\x1b8");
	}
	fn get_colour(c: Colour)->String{ format!("\x1b[{}m",c as usize) }
	fn set_colour(c: Colour){ Self::write(&Self::get_colour(c)); }
	fn clear_display(c: ClearType){ Self::write(&format!("\x1b[{}J",c as usize)); }
	fn clear_line(c: ClearType){ Self::write(&format!("\x1b[{}K",c as usize)); }
	fn show_cur(){ Self::write("\x1b[?25h"); }
	fn hide_cur(){ Self::write("\x1b[?25l"); }
}

macro_rules! get_def_colour{($settings: expr, $state: expr, $focused_state: pat) => {[
	$settings.background,
	$settings.modifier,
	match $state{$focused_state => $settings.focused, _ => $settings.blured, },
]};}
macro_rules! get_focus_colour{($settings: expr, $state: expr, $focused_state: pat) => {match $state{
	$focused_state => [$settings.active_focused, $settings.active_focused_background, $settings.active_focused_modifier],
	_ => [$settings.active_blured, $settings.active_blured_background, $settings.active_blured_modifier],
}};}

enum TodoState{
	Todo,
	Doing,
	Done,
}

struct Todo{
	pub children: Vec<Todo>,
	pub name: String,
	pub description: String,
	pub state: TodoState,
	pub open: bool,
}
impl Todo{
	fn new(name:String)->Self{Todo {
		children: Vec::new(),
		name: name,
		description: String::new(),
		state: TodoState::Todo,
		open: false,
	}}
	fn draw(&self, indent: usize, selected: &Todo, width: usize, default_colour:&[Colour], select_colour:&[Colour]){
		IO::clear_line(ClearType::FromCur);
		if std::ptr::eq(self, selected) {for c in select_colour{IO::set_colour((*c).clone());}}
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
		if self.name.chars().count() > width - loc{
			IO::write(&self.name.replace('\r', " ").chars().take(width - loc - 1).collect::<String>());
			IO::write("…");
		}else{IO::write(self.name.replace('\r', " ").as_str());}
		IO::write("\n");
		if std::ptr::eq(self, selected) {for c in default_colour{IO::set_colour((*c).clone());}}
		if self.open{for child in &self.children{ child.draw(indent+1, selected, width, default_colour, select_colour); }}
	}
	fn draw_details(
		&self,
		start: (usize, usize),
		width: usize,
		height: usize,
		settings: &Settings,
		state: &State,
		scroll_offset: &mut usize,
	) -> Option<(usize,usize)> {
		let width = width - 1;

		let i = RefCell::new(0);

		let content_break = format!("{}{}",IO::get_colour(settings.ui_elements),"=".repeat(width));

		let mut writeln: Box<dyn FnMut(&str)> = Box::new(|line|{
			if *i.borrow() >= *scroll_offset && *i.borrow() - *scroll_offset < height{
				IO::move_cur(start.0, start.1+*i.borrow() - *scroll_offset);
				IO::write(line);
			}
			// log(&format!("got to line {}, height is {}\n",*i.borrow(),height));
			i.replace_with(|i|{*i+1});
		});

		let mut rv = None;

		let mut def_colour = get_def_colour!(settings,state,State::Name(_));
		for c in def_colour{IO::set_colour(c);}

		write_str_with_width(&self.name, width, &mut writeln);
		if let State::Name(j) = state{
			rv = Some(get_curs_pos(&self.name, start, width, *j));
		}
		writeln(&content_break);
		
		def_colour = get_def_colour!(settings,state,State::Description(_));
		for c in def_colour{IO::set_colour(c);}

		if let State::Description(j) = state{
			rv = Some(get_curs_pos(&self.description, (start.0, start.1+*i.borrow()), width, *j));
		}
		write_str_with_width(&self.description, width, &mut writeln);
		writeln(&content_break);

		IO::set_colour(settings.ui_elements);
		let content_height = *i.borrow();
		let mut scroll_top = 0;
		let mut scroll_height = height;
		if content_height > height{ // i.e. no div by 0
			scroll_height = ((height as f32 / content_height as f32) * height as f32) as usize;
			scroll_top = ((*scroll_offset as f32 / (content_height - height) as f32) * (height - scroll_height) as f32) as usize;
			draw_vertical_line(height, start.0+width, 1, '║');
		}
		draw_vertical_line(scroll_height, start.0+width, scroll_top+1, '█');
		return rv;
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
	Insert,
	Delete,
	CtrlUp,
	CtrlDown,
	CtrlLeft,
	CtrlRight,
	CtrlDelete,
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
			82=>ControlChar::Insert,
			83=>ControlChar::Delete,
			141=>ControlChar::CtrlUp,
			145=>ControlChar::CtrlDown,
			115=>ControlChar::CtrlLeft,
			116=>ControlChar::CtrlRight,
			147=>ControlChar::CtrlDelete,
			chr=>ControlChar::Unknown(chr),
		}))},Err(e) => { return Err(e); }}
	}else{ return Ok(TermChar::Unknown(chr)); }
}},Err(e) => { return Err(e); }}}
#[derive(Debug)]
enum State{
	Tree,
	Name(usize),
	Description(usize),
}
impl State{fn next(&mut self){*self = match self{
	State::Tree => State::Name(0),
	State::Name(_) => State::Description(0),
	State::Description(_) => State::Tree,
}}}
struct Settings{
	active_focused: Colour,
	active_focused_background: Colour,
	active_focused_modifier: Colour,

	active_blured: Colour,
	active_blured_background: Colour,
	active_blured_modifier: Colour,

	focused: Colour,

	blured: Colour,

	ui_elements: Colour,

	background: Colour,
	modifier: Colour,
}
impl Settings{fn new()->Self{Settings{
	active_focused: Colour::ForegroundBlack,
	active_focused_background: Colour::BrightBackgroundWhite,
	active_focused_modifier: Colour::NoUnderline,
	
	active_blured: Colour::BrightForegroundBlack,
	active_blured_background: Colour::BackgroundWhite,
	active_blured_modifier: Colour::NoUnderline,
	
	focused: Colour::BrightForegroundWhite,

	blured: Colour::ForegroundWhite,

	ui_elements: Colour::ForegroundYellow,
	
	background: Colour::BackgroundBlack,
	modifier: Colour::NoUnderline,
}}}
struct Todos{
	data: Vec<Todo>,
	selected: Vec<usize>,
	settings: Settings,
	state: State,
	scroll_ofset: usize,
}
macro_rules! get_ptr{
	($todos: expr/*Vec<Todo>*/, $id: expr/*Vec<usize>*/)=>{{
		let mut curr = &($todos[$id[0]]);
		for id_el in $id.iter().skip(1){curr = &curr.children[*id_el];}
		curr
	}};
	($self: ident, $id: expr/*Vec<usize>*/)=>{get_ptr!($self.data,$id)};
	($self: ident)=>{get_ptr!($self.data,$self.selected)};
}
macro_rules! get_mut_ptr{
	($todos: expr/*Vec<Todo>*/, $id: expr/*Vec<usize>*/)=>{{
		let mut curr = &mut ($todos[$id[0]]);
		for id_el in $id.iter().skip(1){curr = &mut curr.children[*id_el];}
		curr
	}};
	($self: ident, $id: expr/*Vec<usize>*/)=>{get_mut_ptr!($self.data,$id)};
	($self: ident)=>{get_mut_ptr!($self.data,$self.selected)};
}
macro_rules! get_parent_arr{
	($todos: expr/*Vec<Todo>*/, $id: expr/*Vec<usize>*/)=>{
		if $id.len() > 1 {&get_ptr!($todos, &$id[..$id.len()-1]).children } else {&$todos}
	};
	($self: ident, $id: expr/*Vec<usize>*/)=>{get_parent_arr!($self.data,$id)};
	($self: ident)=>{get_parent_arr!($self.data,$self.selected)};
}
macro_rules! get_mut_parent_arr{
	($todos: expr/*Vec<Todo>*/, $id: expr/*Vec<usize>*/)=>{
		if $id.len() > 1 { &mut get_mut_ptr!($todos, &$id[..$id.len()-1]).children } else {&mut $todos}
	};
	($self: ident, $id: expr/*Vec<usize>*/)=>{get_mut_parent_arr!($self.data,$id)};
	($self: ident)=>{get_mut_parent_arr!($self.data,$self.selected)};
}
macro_rules! get_text{($todos: ident) => {match $todos.state{
	State::Name(_) => &get_ptr!($todos).name,
	State::Description(_) => &get_ptr!($todos).description,
	_=>panic!(),
}}; }
macro_rules! get_mut_text{($todos: ident) => {match $todos.state{
	State::Name(_) => &mut get_mut_ptr!($todos).name,
	State::Description(_) => &mut get_mut_ptr!($todos).description,
	_=>panic!(),
}}; }
macro_rules! IF_TYPING{($todos: ident, $code: block) => {
	if let State::Name(_) | State::Description(_) = $todos.state {} else{return;}
	$code
}; }
macro_rules! IF_NOT_TYPING{($todos: ident, $code: block) => {
	if let State::Tree = $todos.state {} else{return;}
	$code
}; }
macro_rules! IF_TYPING_I{($todos: ident, $i: pat, $code: block) => {
	if let State::Name($i)| State::Description($i) = $todos.state $code
}; }

impl Todos{
	fn update_state(&mut self){IF_NOT_TYPING!(self, {
		get_mut_ptr!(self).state = match get_ptr!(self).state{
			TodoState::Done => TodoState::Todo,
			TodoState::Todo => TodoState::Doing,
			TodoState::Doing => TodoState::Done,
		}
	});}
	fn draw(&mut self, (width, height): (usize,usize)){
		IO::move_cur(1,1);
		let curr = get_ptr!(self);
		let def_colour = get_def_colour!(self.settings,self.state,State::Tree);
		let focus_colour = get_focus_colour!(self.settings,self.state,State::Tree);
		for c in def_colour{IO::set_colour(c);}
		for todo in &self.data{ todo.draw(0, curr, (width/2)-1, &def_colour, &focus_colour); }
		IO::clear_display(ClearType::FromCur); // we dont clear the whole screen before redrawing as that causes flicker instead it is done one line at a time
		
		for c in def_colour{IO::set_colour(c);}
		IO::set_colour(self.settings.ui_elements);
		draw_vertical_line(height, width/2, 1, '│');
		let detail_width = width/2;
		let detail_start = (width/2)+1;
		let positions = curr.draw_details((detail_start,1), detail_width, height, &self.settings, &self.state, &mut self.scroll_ofset);
		match self.state{
			State::Tree => IO::hide_cur(),
			State::Description(_) | State::Name(_) => {
				IO::show_cur();
				let positions = positions.unwrap();
				IO::move_cur(positions.0, positions.1);
			},
		}
		IO::flush();
	}
	fn select_prev(&mut self){IF_NOT_TYPING!(self, {
		if self.selected.last() == Some(&0) {
			if self.selected.len() > 1 { self.selected.pop(); }
		} else {
			*self.selected.last_mut().unwrap() -= 1;
			let curr = get_ptr!(self);
			if curr.open && curr.children.len() > 0{
				self.selected.push(curr.children.len()-1);
			}
		}
	});}
	fn select_next(&mut self){IF_NOT_TYPING!(self,{
		let curr = get_ptr!(self);
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
	});}
	fn close_sel(&mut self){IF_NOT_TYPING!(self,{get_mut_ptr!(self).open = false;});}
	fn open_sel(&mut self){IF_NOT_TYPING!(self,{get_mut_ptr!(self).open = true;});}
	fn move_sel_down(&mut self){IF_NOT_TYPING!(self,{
		// get_mut_ptr!(self).open = false;
		let index = *self.selected.last().unwrap();
		let next_el = get_parent_arr!(self).get(index+1);
		if next_el.is_some() {
			if next_el.unwrap().open{
				let item = get_mut_parent_arr!(self).remove(index);
				get_mut_ptr!(self).children.insert(0, item);
				self.selected.push(0);
			}else{
				get_mut_parent_arr!(self).swap(index, index+1);
				*self.selected.last_mut().unwrap() += 1;
			}
		}else if self.selected.len() > 1{
			let item = get_mut_parent_arr!(self).remove(index);
			self.selected.pop();
			*self.selected.last_mut().unwrap() += 1;
			get_mut_parent_arr!(self).insert(
				*self.selected.last().unwrap(), item
			);
		}
	});}
	fn move_sel_up(&mut self){IF_NOT_TYPING!(self,{
		// get_mut_ptr!(self).open = false;
		if self.selected.len() > 1 && self.selected.last() == Some(&0){
			self.selected.pop();
			let item = get_mut_ptr!(self).children.remove(0);

			get_mut_parent_arr!(self).insert(
				*self.selected.last().unwrap(), item
			);
		}else if self.selected.last().unwrap() > &0{
			let index = *self.selected.last().unwrap();
			if get_parent_arr!(self)[index-1].open{
				let parent = get_mut_parent_arr!(self);
				*self.selected.last_mut().unwrap() -= 1;
				self.selected.push(parent[index-1].children.len());
				let item = parent.remove(index);
				parent[index-1].children.push(item);
			}else{
				get_mut_parent_arr!(self).swap(index, index-1);
				*self.selected.last_mut().unwrap() -= 1;
			}
		}
	});}
	fn is_sel_valid(&self)->bool{
		let mut curr = &self.data;
		for i in &self.selected{
			if *i >= curr.len(){ return false; }
			curr = &curr[*i].children;
		}
		return true;
	}
	fn add_todo(&mut self){
		self.selected = vec!(self.data.len());
		self.data.push(Todo::new(String::new()));
	}
	fn remove_todo(&mut self){
		let id = self.selected.last().unwrap();
		get_mut_parent_arr!(self).remove(*id);
		if *id > 0{ *self.selected.last_mut().unwrap()-=1; }
		else if self.selected.len() > 1 {self.selected.pop();}
		else if self.data.len() == 0 {self.add_todo();}
	}
	fn try_update(&mut self, chr: char){IF_TYPING!(self,{
		let len = get_text!(self).len();
		IF_TYPING_I!(self, i, {
			get_mut_text!(self).insert(len-i, chr);
		});
	});}
	fn try_backspace(&mut self){IF_TYPING_I!(self, i, {
		let text = get_mut_text!(self);
		let len = text.chars().count();
		if i < len{text.remove(text.char_indices().nth(len-i-1).unwrap().0);}
	});}
	fn try_backspace_word(&mut self){IF_TYPING!(self,{
		let mut j = 0;
		let text = get_text!(self);
		let len =  text.chars().count();
		if len > 0 {IF_TYPING_I!(self, i, {
			let i = len-i;
			if text.chars().nth(i-1) == Some('\r'){
				self.try_backspace();
				return;
			}
			while i>j && text.chars().nth(i-j-1) == Some(' '){ j+=1; }
			while i>j && {
				let chr = text.chars().nth(i-j-1);
				chr.is_some() && chr != Some(' ') && chr != Some('\r')
			}{ j+=1; }
		});}
		for _ in 0..j { self.try_backspace(); }
	});}
	fn try_move_curs_left(&mut self){IF_TYPING!(self,{
		let len = get_text!(self).len();
		IF_TYPING_I!(self, ref mut i, { if *i < len { *i+=1; } });
	});}
	fn try_move_curs_right(&mut self){IF_TYPING_I!(self, ref mut i, {
		if *i > 0 { *i-=1; }
	});}
	fn try_move_curs_home(&mut self){IF_TYPING!(self,{
		let len = get_text!(self).len();
		IF_TYPING_I!(self, ref mut i, { *i=len; });
	});}
	fn try_move_curs_end(&mut self){IF_TYPING_I!(self, ref mut i, { *i=0; });}
}

fn draw_vertical_line(height: usize, col: usize, row: usize, chr: char){ for i in row..row+height{
	IO::move_cur(col, i);
	IO::write(&chr.to_string());
}}
fn write_str_with_width<F>(text: &String, width: usize, writeln: &mut F) where F: FnMut(&str){
	for text in text.split('\r'){
		let text_chrs = &mut text.chars();
		loop{
			let line: Vec<char> = text_chrs.take(width).collect();
			let should_break = line.len() < width;
			writeln(&String::from_iter(line));
			if should_break{break;}
		}
	}
}
fn get_curs_pos(text: &String, start: (usize, usize), width: usize, pos: usize) -> (usize, usize){
	//TODO: make this work with chars not bytes``
	let mut i = 0;
	let text: String = text.chars().take(text.chars().count()-pos).collect();
	let mut curr = start;
	for text in text.split('\r'){
		let mut text_len = text.chars().count();
		while text_len > width{
			text_len -= width;
			i+=1;
		}
		curr = (start.0, start.1+i);
		curr.0 += text_len;
		if text_len == width{
			curr = (start.0, start.1+i+1);
		}
		i+=1;
	}
	return curr;
}

fn todo_loop(mut todos: Todos){
	IO::set_up_screen();
	if todos.data.len() == 0{todos.data.push(Todo::new("Add Todos".to_string()).into())}
	if !todos.is_sel_valid(){todos.selected = vec!(0);}
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
				'\r' | ' '=>match todos.state {
					State::Tree => todos.update_state(),
					State::Name(_) | State::Description(_) => todos.try_update(chr),
				},
				'\t'=>todos.state.next(),
				'+'=>match todos.state {
					State::Tree => todos.add_todo(),
					State::Name(_) | State::Description(_) => todos.try_update(chr),
				},
				'\x1b'=>break, // this is esc
				'\u{7f}'=>todos.try_backspace_word(),// ctrl backspace
				'\u{8}'=>todos.try_backspace(),
				'\u{9c}'=>todos.try_update('£'),
				chr=>if chr >= ' ' && chr <= 126 as char {todos.try_update(chr)},
			},
			TermChar::ControlChar(chr) => match chr{
				ControlChar::Up=>match todos.state {
					State::Tree => todos.select_prev(),
					State::Name(_) | State::Description(_) => if todos.scroll_ofset > 0 { todos.scroll_ofset-=1 },
				},
				ControlChar::Down=>match todos.state {
					State::Tree => todos.select_next(),
					State::Name(_) | State::Description(_) => todos.scroll_ofset+=1,
				},
				ControlChar::Left=>match todos.state {
					State::Tree => todos.close_sel(),
					State::Name(_) | State::Description(_) => todos.try_move_curs_left(),
				},
				ControlChar::Right=>match todos.state {
					State::Tree => todos.open_sel(),
					State::Name(_) | State::Description(_) => todos.try_move_curs_right(),
				},
				ControlChar::CtrlDown=>todos.move_sel_down(),
				ControlChar::CtrlUp=>todos.move_sel_up(),
				ControlChar::Delete=>match todos.state {
					State::Tree => todos.remove_todo(),
					State::Description(ref mut i) | State::Name(ref mut i) => {if *i > 0 {
						*i-=1;
						todos.try_backspace();
					}},
				},
				ControlChar::Home=>todos.try_move_curs_home(),
				ControlChar::End=>todos.try_move_curs_end(),
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

fn main() {todo_loop(Todos{
	data:vec!(Todo{
		children: vec!(
			Todo::new("child1".to_string()),
			Todo::new("child2".to_string()),
		),
		name: "parent".to_string(),
		description: "this is the description".to_string(),
		state: TodoState::Doing,
		open: true,
	},Todo::new("parent2".to_string()),),
	selected:vec!(0),
	settings:Settings::new(),
	state: State::Tree,
	scroll_ofset: 0,
});}
