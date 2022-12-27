use walkdir::WalkDir;
use std::path::Path;
use std::io;
use fs_extra::dir::get_size;
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use fltk::{prelude::*, *};
use fltk_evented::Listener;
use fltk::button::Button;
use fltk_theme::{WidgetScheme,SchemeType};
use fltk::{app, enums::FrameType};
use fltk_table::{SmartTable, TableOpts};
use std::fs::{metadata, remove_dir_all};
use rand::Rng;
use std::fs::File;
use std::io::{BufWriter, Write};
use fltk::{printer,text};

const SEARCH: &str = r#"
<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" fill="currentColor" class="bi bi-search" viewBox="0 0 16 16">
  <path d="M11.742 10.344a6.5 6.5 0 1 0-1.397 1.398h-.001c.03.04.062.078.098.115l3.85 3.85a1 1 0 0 0 1.415-1.414l-3.85-3.85a1.007 1.007 0 0 0-.115-.1zM12 6.5a5.5 5.5 0 1 1-11 0 5.5 5.5 0 0 1 11 0z"/>
</svg>
"#;

const RELOAD: &str = r#"
<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" fill="currentColor" class="bi bi-arrow-clockwise" viewBox="0 0 16 16">
  <path fill-rule="evenodd" d="M8 3a5 5 0 1 0 4.546 2.914.5.5 0 0 1 .908-.417A6 6 0 1 1 8 2v1z"/>
  <path d="M8 4.466V.534a.25.25 0 0 1 .41-.192l2.36 1.966c.12.1.12.284 0 .384L8.41 4.658A.25.25 0 0 1 8 4.466z"/>
</svg>
"#;

//return vector of subdirectories
fn getdir (dir_path:String,mindepth:i32,maxdepth:i32)->Vec<walkdir::DirEntry>{   
    let mut vec : Vec<walkdir::DirEntry> = Vec::new();

        for entry in WalkDir::new(dir_path.clone()).min_depth(mindepth.try_into().unwrap()).max_depth(maxdepth.try_into().unwrap()) {
        match entry {
            Ok(entry) =>    {let start = metadata(dir_path.clone()).unwrap();
                if start.is_dir(){
                    vec.push(entry)}},
            Err(err) => {
                let path = err.path().unwrap_or(Path::new("")).display();
                println!("failed to access entry {}", path);
                if let Some(inner) = err.io_error() {
                    match inner.kind() {
                        io::ErrorKind::InvalidData => {
                            println!(
                                "entry contains invalid data: {}",
                                inner)
                        }
                        io::ErrorKind::PermissionDenied => {
                            println!(
                                "Missing permission to read entry: {}",
                                inner)
                        }
                        _ => {
                            println!(
                                "Unexpected error occurred: {}",
                                inner)
                        }
                    }
                }
            }
        }
    }

    vec

}
//percentage calculation of a certain directory
fn percentage (dir_path:String)->Vec<Decimal>{
    let mut percent : Vec<Decimal> = Vec::new();
    let mut dir : Vec<walkdir::DirEntry> = Vec::new();
    let mut size : Vec<u64> = Vec::new();
    dir = getdir(dir_path.to_string(),0,1);
    size = getsize(dir);
    let sum = Decimal::new(size[0].try_into().unwrap(), 0);
    percent.push(Decimal::new(100, 0));

    for i in 1..size.len(){
        let siz = Decimal::new(size[i].try_into().unwrap(), 0);
        let perc = Decimal::new(100, 0);
        let calc = (siz * perc/sum).round_dp(1);    
    	percent.push(calc);

    }
    percent
}

//get the size of each directory
fn getsize(vec:Vec<walkdir::DirEntry>)->Vec<u64>{
    let mut size : Vec<u64> = Vec::new();
    for i in vec {
    	size.push(get_size(i.path()).unwrap());
    //	println!("{},{}", i.path().display(), get_size(i.path()).unwrap());
    }
    
    //println!("{}", size[4]);
    size
}
//get the size of each directory in GB/MB/KB/B
fn size(vec:Vec<walkdir::DirEntry>)->Vec<String>{
    let mut size : Vec<u64> = Vec::new();
    let mut size_str : Vec<String> = Vec::new();
    size = getsize(vec);
    for i in size{
        if i > 1000000000 //GB
        {
            let n = i/1000000000 ;
            size_str.push(n.to_string()+" GB");
        }
        else if i > 1000000//MB
        {
            let n = i/1000000 ;
            size_str.push(n.to_string()+" MB");
        }
        else if i > 1000//KB
        {
            let n = i/1000 ;
            size_str.push(n.to_string()+" KB");
        }
        else
        {
            size_str.push(i.to_string()+" B");
        }
        

    }
    //println!("{}", size[4]);
    size_str
}
//delete directory
fn deldir(dir_path:String) {
    remove_dir_all(&dir_path);
}
//write the printed file
fn fileWrite(fileName:String,vec:Vec<walkdir::DirEntry>,vec2:Vec<Decimal>,size:Vec<String>) 
{
    let f = File::create(fileName).expect("Unable to create file");
    let mut f = BufWriter::new(f);
    f.write_all(&"FileName  ---------->  Size(%)  ---------->  Size".as_bytes()).expect("Unable to write data");
    f.write_all(&"\n".as_bytes()).expect("Unable to write data");
    f.write_all(&"\n".as_bytes()).expect("Unable to write data");
    for i in 0..vec.len()
    {    
        f.write_all(&vec[i].file_name().to_string_lossy().as_bytes()).expect("Unable to write data");
        f.write(&"  ---------->  ".as_bytes()).expect("Unable to write data");
        f.write_all(&(vec2[i].to_f64().unwrap().to_string()+" %").as_bytes()).expect("Unable to write data");
        f.write_all(&"  ---------->  ".as_bytes()).expect("Unable to write data");
        f.write_all(&size[i].as_bytes()).expect("Unable to write data");
        f.write_all(&"\n".as_bytes()).expect("Unable to write data");
    }
   

}


fn main() {
    let mut dir_path= String::from( "./target/debug");
    let mut percent : Vec<Decimal> = Vec::new();
    let mut history : Vec<String> = Vec::new();
    let mut size_str : Vec<String> = Vec::new();
    percent = percentage(dir_path.to_string());
    let mut vec : Vec<walkdir::DirEntry> = Vec::new();
    let maxdepth=1; let mindepth=0;
    vec = getdir(dir_path.to_string(),mindepth,maxdepth);
    size_str = size(vec.clone());

    let app = app::App::default().with_scheme(app::Scheme::Gtk);
    let mut win = window::Window::new(100, 100, 850, 800, "Disk Analyzer").center_screen(); 
    let mut menu:Listener<_> = menu::MenuBar::new(5, 1, 200, 20, None).into();
    let mut glut_win = window::Window::new(5, 60, win.w() - 600, 700, None);
    glut_win.set_color(enums::Color::White);
    glut_win.set_frame(FrameType::BorderBox);
//for printing
    let mut printable = text::TextDisplay::default();
    printable.set_frame(FrameType::NoBox);
    printable.set_scrollbar_size(0);


    menu.add_choice("File/Load folder...|File/Print|File/Quit|Help/About");
    menu.set_frame(FrameType::FlatBox);
    menu.set_text_size(14);

    //table setup
    let mut table = SmartTable::default()
    .with_size(450, 690)
    .with_pos(5, 20)
    .with_opts(TableOpts {
        rows: vec.len() as i32,    //number of sub directories
        cols: 3,
        editable: false,
        header_frame: FrameType::NoBox,
        header_color: enums::Color::White,
        cell_border_color: enums::Color::White,
        cell_align:enums::Align::Left,
        header_align:enums::Align::Left,
        cell_font_color:enums::Color::Black,
        cell_padding:0,
        header_font:enums::Font::TimesBoldItalic,
        ..Default::default()
    });

    table.set_frame(FrameType::NoBox);
    table.set_color(enums::Color::White);
    table.scrollbar().visible_focus(false);
    table.set_col_header_value(0, "Folder");
    table.set_col_width(0, 200);
    table.set_col_header_value(1, "Size");
    table.set_col_width(1, 110);
    table.set_col_header_value(2, "Percentage");
    table.set_col_width(2,100);

    table.set_col_header_height(30);
    table.set_row_height(0, 40);
    
    glut_win.end();
    
    for i in 0..vec.len() {
        if i==0{
            table.set_cell_value(0,0, &vec[i].file_name().to_string_lossy().into_owned());
        }
        else {
            let blank = "  ".to_owned();
            let emp= vec[i].file_name().to_string_lossy().into_owned();
            let new = blank + &emp;
            table.set_cell_value(i.try_into().unwrap(),0, &new);
        }

        table.set_cell_value(i.try_into().unwrap(),1, &size_str[i]);
        table.set_cell_value(i.try_into().unwrap(),2, &(percent[i].to_f64().unwrap().to_string()+"%"));
        table.set_row_header_value(i.try_into().unwrap(),"");
    }

    let mut chart = misc::Chart::default().with_size(580, 700).with_pos(270, 60);
    let mut inp1 = input::Input::new(520,765,130,30,"Starting directory ");
    inp1.set_label("Starting directory ");
    let mut enter : Listener<_> = Button::default().into();
    enter.set_image(Some(image::SvgImage::from_data(SEARCH).unwrap()));
    enter.set_size(20, 30);
    enter.set_pos(650, 765);

    //Chart setup
    win.make_resizable(true);
    chart.set_type(misc::ChartType::Pie);
    chart.set_bounds(0.0, 100.0);
    chart.set_text_size(18);
//for random colors
    let mut r=rand::thread_rng();
    let mut g=rand::thread_rng();
    let mut b=rand::thread_rng();
    for i in 1..percent.len()
    {
        let R = r.gen_range(0..255);
        let G = g.gen_range(0..255);
        let B = b.gen_range(0..255);
        chart.add(percent[i].to_f64().unwrap(), &vec[i].file_name().to_string_lossy(), enums::Color::from_rgb(R,G,B));

    }

    chart.set_color(enums::Color::White);
    let mut choice: Listener<_>  = menu::Choice::default().with_label("Chart type").into();
    choice.set_pos(515, 8);
    choice.set_size( 200, 40);
    choice.add_choice("Bar | HorzBar | Line | Fill | Spike | Pie | SpecialPie");
    choice.set_value(5);
    choice.set_color(enums::Color::White);

    //buttons
    let mut btn : Listener<_> = Button::default().with_label("move").into();
    let mut back : Listener<_> = Button::default().with_label("Back").into();
    let mut delete : Listener<_> = Button::default().with_label("Delete").into();
    let mut reload : Listener<_> = Button::default().into();
    back.set_size(60, 30);
    back.set_pos(270, 760);
    btn.set_size(60, 30);
    btn.set_pos(195, 760);
    delete.set_size(60, 30);
    delete.set_pos(5, 760);
    reload.set_image(Some(image::SvgImage::from_data(RELOAD).unwrap()));
    reload.set_size(20, 25);
    reload.set_pos(5, 35);

  win.end();
  win.show();


    while app.wait() {
//move to another directory
        if btn.triggered() {
            inp1.set_label_color(enums::Color:: Black);
            inp1.redraw_label();
            let c = table.callback_row() as usize;
            let mut temp : Vec<walkdir::DirEntry> = Vec::new();
            temp = getdir(vec[c].path().to_string_lossy().into(),mindepth,maxdepth);
            if (temp.len() > 1) && (c!=0) {
                history.push(dir_path.clone());
                dir_path = vec[c].path().to_string_lossy().into();
                percent = percentage(dir_path.to_string());
                vec = temp;
                size_str = size(vec.clone());
                chart.clear();
                
                for i in 1..percent.len()
                {
                    let R = r.gen_range(0..255);
                    let G = g.gen_range(0..255);
                    let B = b.gen_range(0..255);
                    chart.add(percent[i].to_f64().unwrap(), &vec[i].file_name().to_string_lossy(), enums::Color::from_rgb(R,G,B));
                }
                table.clear();
                
                if (table.rows() as usize) < vec.len(){
                    for i in table.rows() as usize..vec.len() {
                        table.append_empty_row(" ");
        
                    }
                }
    
                table.set_rows(vec.len() as i32);
                println!("here");
    
                for i in 0..vec.len() {
                    if i==0{
                        table.set_cell_value(0,0, &vec[i].file_name().to_string_lossy().into_owned());
                    }
                    else {
                        let blank = "  ".to_owned();
                        let emp= vec[i].file_name().to_string_lossy().into_owned();
                        let new = blank + &emp;
                        table.set_cell_value(i.try_into().unwrap(),0, &new);
                    }
            
                    table.set_cell_value(i.try_into().unwrap(),1, &size_str[i]);
                    table.set_cell_value(i.try_into().unwrap(),2, &(percent[i].to_f64().unwrap().to_string()+"%"));
                    table.set_row_header_value(i.try_into().unwrap(),"");
    
                }
            }

        }
        // previous directory
        if back.triggered(){
            inp1.set_label_color(enums::Color:: Black);
            inp1.redraw_label();
            if history.len() > 0{
            
            dir_path = history.pop().unwrap();
            println!("{} ", dir_path);
            println!("back");
            println!("{}", history.len());
            percent = percentage(dir_path.to_string());
            vec = getdir(dir_path.to_string(),mindepth,maxdepth);
            size_str = size(vec.clone());
            chart.clear();
            for i in 1..percent.len()
            {
                let R = r.gen_range(0..255);
                let G = g.gen_range(0..255);
                let B = b.gen_range(0..255);
                chart.add(percent[i].to_f64().unwrap(), &vec[i].file_name().to_string_lossy(), enums::Color::from_rgb(R,G,B));
            }
            table.clear();

            table.set_rows(vec.len() as i32);
            println!("here");

            for i in 0..vec.len() {
                if i==0{
                    table.set_cell_value(0,0, &vec[i].file_name().to_string_lossy().into_owned());
                }
                else {
                    let blank = "  ".to_owned();
                    let emp= vec[i].file_name().to_string_lossy().into_owned();
                    let new = blank + &emp;
                    table.set_cell_value(i.try_into().unwrap(),0, &new);
                }
        
                table.set_cell_value(i.try_into().unwrap(),1, &size_str[i]);
                table.set_cell_value(i.try_into().unwrap(),2, &(percent[i].to_f64().unwrap().to_string()+"%"));
                table.set_row_header_value(i.try_into().unwrap(),"");

            }
        }
        }
        //input path
        if enter.triggered() { /////////////////////////////////////////////////table needed here
            inp1.set_label_color(enums::Color:: Black);
            inp1.redraw_label();
            let mut temp : Vec<walkdir::DirEntry> = Vec::new();
            temp = getdir(inp1.value().to_string(),mindepth,maxdepth);
            println!("{}",temp.len());
            if (temp.len() > 0) && inp1.value().to_string() != dir_path{
                history.push(dir_path.clone());
                dir_path = inp1.value().to_string();
                println!("{}", dir_path);
                println!("forward");
                println!("{}", history.len());
                percent = percentage(dir_path.to_string());
                vec=temp;
                size_str = size(vec.clone());
                chart.clear();
                for i in 1..percent.len()
                {
                    let R = r.gen_range(0..255);
                    let G = g.gen_range(0..255);
                    let B = b.gen_range(0..255);
                    chart.add(percent[i].to_f64().unwrap(), &vec[i].file_name().to_string_lossy(), enums::Color::from_rgb(R,G,B));
                }

                if (table.rows() as usize) < vec.len(){
                    for i in table.rows() as usize..vec.len() {
                        table.append_empty_row(" ");
                    }
                }
                table.clear();
                table.set_rows(vec.len() as i32);

                println!("here");
    
                for i in 0..vec.len() {
                    if i==0{
                        table.set_cell_value(0,0, &vec[i].file_name().to_string_lossy().into_owned());
                    }
                    else {
                        let blank = "  ".to_owned();
                        let emp= vec[i].file_name().to_string_lossy().into_owned();
                        let new = blank + &emp;
                        table.set_cell_value(i.try_into().unwrap(),0, &new);
                    }
            
                    table.set_cell_value(i.try_into().unwrap(),1, &size_str[i]);
                    table.set_cell_value(i.try_into().unwrap(),2, &(percent[i].to_f64().unwrap().to_string()+"%"));
                    table.set_row_header_value(i.try_into().unwrap(),"");
    
                }
            }
            else if inp1.value().to_string() != dir_path{
                inp1.set_label_color(enums::Color::Red);
                inp1.redraw_label();
            }

        }

        if choice.triggered(){
            chart.set_type(misc::ChartType::from_i32(choice.value()));
            chart.redraw();
        }
        if menu.triggered(){
            if let Some(choice) = menu.choice() {
                match choice.as_str() {
                    "Load folder..." => {
                        let mut dlg = dialog::NativeFileChooser::new(dialog::NativeFileChooserType::BrowseDir);
                        dlg.show();
                        println!("{:?}", dlg.filename());
                        if dlg.filename().into_os_string().into_string().unwrap() != ""{
                            history.push(dir_path.clone());
                            let dir_path = dlg.filename().into_os_string().into_string().unwrap();
                            println!("{}", dir_path);
                            println!("forward");
                            println!("{}", history.len());
                            percent = percentage(dir_path.to_string());
                            vec = getdir(dir_path,mindepth,maxdepth);
                            size_str = size(vec.clone());
                            chart.clear();
                            for i in 1..percent.len()
                            {
                                let R = r.gen_range(0..255);
                                let G = g.gen_range(0..255);
                                let B = b.gen_range(0..255);
                                chart.add(percent[i].to_f64().unwrap(), &vec[i].file_name().to_string_lossy(), enums::Color::from_rgb(R,G,B));
                            }
                            table.clear();
            
                            if (table.rows() as usize) < vec.len(){
                                for i in table.rows() as usize..vec.len() {
                                    table.append_empty_row(" ");
                    
                                }
                            }
                
                            table.set_rows(vec.len() as i32);
                            println!("here");
                
                            for i in 0..vec.len() {
    
                                table.set_cell_value(i.try_into().unwrap(),0, &vec[i].file_name().to_string_lossy().into_owned());
                                table.set_cell_value(i.try_into().unwrap(),1, &size_str[i]);
                                table.set_cell_value(i.try_into().unwrap(),2, &(percent[i].to_f64().unwrap().to_string()+"%"));
                                table.set_row_header_value(i.try_into().unwrap()," ");
                
                            }
                        }
                        else {
                            percent = percentage(dir_path.to_string());
                            println!("{}",dir_path);
                            vec = getdir(dir_path.to_string(),mindepth,maxdepth);
                            size_str = size(vec.clone());
                            chart.clear();
                            for i in 1..percent.len()
                            {
                                let R = r.gen_range(0..255);
                                let G = g.gen_range(0..255);
                                let B = b.gen_range(0..255);
                                chart.add(percent[i].to_f64().unwrap(), &vec[i].file_name().to_string_lossy(), enums::Color::from_rgb(R,G,B));
                            }
                            table.clear();

                            if (table.rows() as usize) < vec.len(){
                                for i in table.rows() as usize..vec.len() {
                                    table.append_empty_row(" ");
                    
                                }
                            }
                
                            table.set_rows(vec.len() as i32);
                            println!("here");

                            for i in 0..vec.len() {
                                if i==0{
                                    table.set_cell_value(0,0, &vec[i].file_name().to_string_lossy().into_owned());
                                }
                                else {
                                    let blank = "  ".to_owned();
                                    let emp= vec[i].file_name().to_string_lossy().into_owned();
                                    let new = blank + &emp;
                                    table.set_cell_value(i.try_into().unwrap(),0, &new);
                                }
                        
                                table.set_cell_value(i.try_into().unwrap(),1, &size_str[i]);
                                table.set_cell_value(i.try_into().unwrap(),2, &(percent[i].to_f64().unwrap().to_string()+"%"));
                                table.set_row_header_value(i.try_into().unwrap(),"");

                            }
                        }

                    }
                    "Print" => {
                        let mut printer = printer::Printer::default();
                            if printer.begin_job(0).is_ok() {
                                let (w, h) = printer.printable_rect();
                                printable.set_size(w - 40, h - 40);
                                // Needs cleanup
                                fileWrite((&"FLTK.pdf").to_string(),vec.clone(), percent.clone(), size_str.clone());
                                let line_count = printable.count_lines(0, printable.buffer().unwrap().length(), true) / 45;
                                for i in 0..=line_count {
                                    printable.scroll(45 * i, 0);
                                    printer.begin_page().ok();
                                    printer.print_widget(&printable, 20, 20);
                                    printer.end_page().ok();
                                    println!("here");
                                }
                                printer.end_job();
                            }
                    }
    
                    "Quit" => app::quit(),
                    "About" => dialog::alert_default("This is a disk analyzer for the Linux operating system designed by a group in the Operating Systems course in the American University in Cairo. 
                    Developers' Information:If you are interested in sharing with us any of your updates, or you are having a trouble, please contact any of these email addresses below:
                    shaalan@aucegypr.edu
                    mariam2000@aucegypt.edu
                    salmaemad@aucegypt.edu"),
                    _ => (),
                }
            }
        }
        if delete.triggered(){
            inp1.set_label_color(enums::Color:: Black);
            inp1.redraw_label();
            let c = table.callback_row() as usize;
            deldir(vec[c].path().to_string_lossy().into());
            percent = percentage(dir_path.to_string());
            vec = getdir(dir_path.to_string(),mindepth,maxdepth);
            size_str = size(vec.clone());
            chart.clear();
            for i in 1..percent.len()
            {
                let R = r.gen_range(0..255);
                let G = g.gen_range(0..255);
                let B = b.gen_range(0..255);
                chart.add(percent[i].to_f64().unwrap(), &vec[i].file_name().to_string_lossy(), enums::Color::from_rgb(R,G,B));
            }
            table.clear();

            table.set_rows(vec.len() as i32);
            println!("here");

            for i in 0..vec.len() {
                if i==0{
                    table.set_cell_value(0,0, &vec[i].file_name().to_string_lossy().into_owned());
                }
                else {
                    let blank = "  ".to_owned();
                    let emp= vec[i].file_name().to_string_lossy().into_owned();
                    let new = blank + &emp;
                    table.set_cell_value(i.try_into().unwrap(),0, &new);
                }
        
                table.set_cell_value(i.try_into().unwrap(),1, &size_str[i]);
                table.set_cell_value(i.try_into().unwrap(),2, &(percent[i].to_f64().unwrap().to_string()+"%"));
                table.set_row_header_value(i.try_into().unwrap(),"");
            }
        }
        if reload.triggered(){
            inp1.set_label_color(enums::Color:: Black);
            inp1.redraw_label();

            percent = percentage(dir_path.to_string());
            vec = getdir(dir_path.to_string(),mindepth,maxdepth);
            size_str = size(vec.clone());
            chart.clear();
            for i in 1..percent.len()
            {
                let R = r.gen_range(0..255);
                let G = g.gen_range(0..255);
                let B = b.gen_range(0..255);
                chart.add(percent[i].to_f64().unwrap(), &vec[i].file_name().to_string_lossy(), enums::Color::from_rgb(R,G,B));
            }
            table.clear();

            table.set_rows(vec.len() as i32);
            println!("here");

            for i in 0..vec.len() {
                if i==0{
                    table.set_cell_value(0,0, &vec[i].file_name().to_string_lossy().into_owned());
                }
                else {
                    let blank = "  ".to_owned();
                    let emp= vec[i].file_name().to_string_lossy().into_owned();
                    let new = blank + &emp;
                    table.set_cell_value(i.try_into().unwrap(),0, &new);
                }
        
                table.set_cell_value(i.try_into().unwrap(),1, &size_str[i]);
                table.set_cell_value(i.try_into().unwrap(),2, &(percent[i].to_f64().unwrap().to_string()+"%"));
                table.set_row_header_value(i.try_into().unwrap(),"");

            }
        }
 
    }

    let widget_scheme = WidgetScheme::new(SchemeType::Aqua);
    widget_scheme.apply();
    app.run().unwrap();
}