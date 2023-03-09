use fltk::{
    app,button::Button, output,
    draw::{set_draw_color, set_line_style, LineStyle, draw_rect, draw_rectf},
    enums::{Color, Event},
    frame::Frame,
    prelude::*,text::TextBuffer,text::TextDisplay, input::FileInput, button::CheckButton ,
    surface::ImageSurface,
    window::Window, image::PngImage, 
};
use std::{cell::{RefCell}};
use std::rc::Rc;
use rfd::FileDialog;
use walkdir::WalkDir;
use csv::Writer;
use std::path::Path;
use std::fs::OpenOptions;
use std::fs::read_to_string;

const WIDTH: i32 = 1000;
const HEIGHT: i32 = 700;
const IM_WIDTH: i32 = 700;
const IM_HEIGHT: i32 = 500;
const BUTTON_WIDTH: i32 = 70;
const BUTON_HEIGHT: i32 = 30;
const BUTON_SPACING: i32 = 20;

#[derive(Clone, Debug)]
pub struct Coords{
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

#[derive(Clone, Debug)]
pub struct ImageAnnotation{
    img: Option<PngImage>,
    width: Option<i32>,
    height: Option<i32>,
    file_name: Option<String>,
    annotations: Option<Coords>
}

impl ImageAnnotation {

    pub fn add_image(&mut self, _file_name: String) {
        
        self.file_name =  Some(_file_name.clone());    

        let mut img = PngImage::load(&_file_name).unwrap();
        img.scale(IM_WIDTH, IM_HEIGHT, true, false);

        self.width = Some(img.width());
        self.height = Some(img.height());
        self.img = Some(img);  
       
    }

}

#[derive(Clone, Debug)]
pub struct Annotator {
    current_pos: Rc<RefCell<usize>>,
    current_img:Rc<RefCell<Option<PngImage>>>,
    current_coords: Rc<RefCell<Option<Coords>>>,
    images: Rc<RefCell<Vec<ImageAnnotation>>>
}

impl Annotator {

    pub fn load_images(&mut self, _paths:  Vec<String>) {

        let mut tmp: Vec<ImageAnnotation> = Vec::new();
        for p in _paths
        {
            let mut a:ImageAnnotation = ImageAnnotation { img: None, width: (None), height: (None), file_name: (None), annotations: (None) };
            a.add_image(p);
            tmp.push(a);
        }
        *self.images.borrow_mut() = tmp;   
    }

    pub fn update_current_coords(&mut self, _coords:  Coords) {

        *self.current_coords.borrow_mut() = Some(_coords);
    }

    pub fn get_current_img(&mut self) -> Option<PngImage>{

        let res = (*self.current_img.borrow_mut()).clone();
        
        return res;

    }

    pub fn get_current_coords(&mut self) -> Option<Coords>{

        let res = (*self.current_coords.borrow_mut()).clone();

        return res;
    }

    pub fn get_coords_by_pos(&mut self) -> Option<Coords>{

        let pos = *self.current_pos.borrow();
        let tmp = (*self.images.borrow_mut())[pos].clone();
        let res = tmp.annotations;

        return res;

    }

    pub fn update_current_img(&mut self) {

        let p =  *self.current_pos.borrow();
        let k = (*self.images.borrow().clone())[p].img.clone();
        
        *self.current_img.borrow_mut() = k;
 
    }


    pub fn add_coords(&mut self) {

        let p =  *self.current_pos.borrow();
        let co = (*self.current_coords.borrow_mut()).clone();

        (*self.images.borrow_mut())[p].annotations = co;

        
    }

    pub fn remove_coords(&mut self) {

        let p =  *self.current_pos.borrow();
        let co = None;

        (*self.images.borrow_mut())[p].annotations = co;

    }

    pub fn increment_pos(&mut self) {

        let size = (*self.images.borrow().clone()).len();
        let pos = *self.current_pos.borrow();

        if pos < size - 1 { 
            *self.current_pos.borrow_mut() += 1;
        }

    }

    pub fn decrement_pos(&mut self) {

        let pos = *self.current_pos.borrow();

        if pos > 0  { 
            *self.current_pos.borrow_mut() -= 1;
        }
        
    }


    pub fn get_current_info_print(&self){

        let curr_img = (*self.current_img.borrow_mut()).clone();

        if !curr_img.is_none(){
            let curr_img = curr_img.unwrap();
            println!("current image [{:?}]: w={:?}, h={:?}", &(*self.current_pos).borrow(), &curr_img.width(), &curr_img.height());
        }

        let curr_coords = (*self.current_coords.borrow_mut()).clone();

        if !(curr_coords.is_none()) {
            let curr_coords = curr_coords.unwrap();
            println!("current coords: x={:?}, y={:?}, w={:?}, h={:?}",&curr_coords.x, &curr_coords.y, &curr_coords.width, &curr_coords.height);
        }

        println!("Added annotations summary:");
        let curr_images = (*self.images.borrow_mut()).clone();

        for im in curr_images {
            let im_coords = im.annotations.clone();
            if !(im_coords.is_none()) {
                let im_coords = im_coords.unwrap();
                
                println!("{:?} | x={:?}, y={:?}, w={:?}, h={:?}",&im.file_name, &im_coords.x, &im_coords.y, &im_coords.width, &im_coords.height);
            }
        }

    }

    pub fn get_current_info_text(&self) -> String{

        let curr_images = (*self.images.borrow_mut()).clone();
        let mut res: String = String::from("");

        for im in curr_images {
            let im_coords = im.annotations.clone();
            if !(im_coords.is_none()) {
                let im_coords = im_coords.unwrap();

                let tmp_txt = format!("{:?} | x={:?}, y={:?}, w={:?}, h={:?} \n\r",&im.file_name.unwrap(), &im_coords.x, &im_coords.y, &im_coords.width, &im_coords.height).replace("\"", "");
                
                res.push_str(&tmp_txt);
            }
        }

        return res;

    }

    pub fn get_current_img_info_text(&self) -> String{

        let pos = *self.current_pos.borrow();
        let size = (*self.images.borrow().clone()).len();
        let all_images = (*self.images.borrow_mut()).clone();
        let curr_image = all_images[pos].clone();
        let img_path = curr_image.file_name.unwrap();
        let filename = Path::new(&img_path).file_name().unwrap();
        let filename = format!("{:?}", filename).replace("\"", "");
        let ids:Vec<&str> = filename.split("_").collect();
        let mut ids_section = String::from("");
        if ids.len()>2{
            let id1 = ids[0];
            let id2 = ids[1];

            ids_section = format!("ID: {} | ID : {} |", id1, id2);
        } 

        let mut res = format!("{}/{} | {} pos: {} width:{}, height: {} | {:?}",pos+1, size, ids_section, pos, curr_image.width.unwrap(), curr_image.height.unwrap(), &filename).replace("\"", "");

        if !curr_image.annotations.is_none() {
            let im_coords = curr_image.annotations.unwrap();
            let coords_txt = format!("| x={:?}, y={:?}, w={:?}, h={:?}",&im_coords.x, &im_coords.y, &im_coords.width, &im_coords.height);

            res = format!("{} {}", res, coords_txt);
         }
        return res;
    }

    pub fn save_csv(&self, _dest: &str, _append:bool) {

        let mut i = 0;

        if _append{
            let file = OpenOptions::new().write(true).append(true).open(&_dest).unwrap();
            let mut csv_writer_metadata = Writer::from_writer(file);
            let curr_images = (*self.images.borrow_mut()).clone();
            for im in curr_images {
                i += 1;
                let im_coords = im.annotations.clone();
                if !(im_coords.is_none()) {
                    let im_coords = im_coords.unwrap();
                    csv_writer_metadata.write_record(&[
                        i.to_string(), 
                        im.file_name.unwrap(), 
                        im.width.unwrap().to_string(), 
                        im.height.unwrap().to_string(), 
                        im_coords.x.to_string(), 
                        im_coords.y.to_string(), 
                        im_coords.width.to_string(), 
                        im_coords.height.to_string()]).ok();
                }
     
            }
    
            csv_writer_metadata.flush().ok();

        } else {

            let file = OpenOptions::new().create(true).write(true).truncate(true).append(false).open(&_dest).unwrap();
            let mut csv_writer_metadata = Writer::from_writer(file);
            csv_writer_metadata.write_record(&["lp", "filename", "img_width", "img_height", "annotation_x", "annotation_y", "annotation_w", "annotation_h"]).ok();
            let curr_images = (*self.images.borrow_mut()).clone();
            for im in curr_images {
                i += 1;
                let im_coords = im.annotations.clone();
                if !(im_coords.is_none()) {
                    let im_coords = im_coords.unwrap();

                    csv_writer_metadata.write_record(&[
                        i.to_string(), 
                        im.file_name.unwrap(), 
                        im.width.unwrap().to_string(), 
                        im.height.unwrap().to_string(), 
                        im_coords.x.to_string(), 
                        im_coords.y.to_string(), 
                        im_coords.width.to_string(), 
                        im_coords.height.to_string()]).ok();
                }
      
            }
    
            csv_writer_metadata.flush().ok();
        }      
  
    }

}


fn main() {
  
    let app = app::App::default().with_scheme(app::Scheme::Gtk);
    let mut wind = Window::new(100, 50, WIDTH, HEIGHT, "Image annotator");
    let mut frame = Frame::default().with_size(IM_WIDTH, IM_HEIGHT).with_pos(0, 0);
    let mut but_select = Button::new(10, HEIGHT - 50, BUTTON_WIDTH, BUTON_HEIGHT, "Select dir");
    let mut but_prev = Button::new(10 + 1 * (BUTTON_WIDTH + BUTON_SPACING), HEIGHT-50, BUTTON_WIDTH, BUTON_HEIGHT, "Prev");
    let mut but_next = Button::new(10 + 2 * (BUTTON_WIDTH + BUTON_SPACING), HEIGHT-50, BUTTON_WIDTH, BUTON_HEIGHT, "Next");
    let mut but_add = Button::new(10 + 3 * (BUTTON_WIDTH + BUTON_SPACING), HEIGHT-50, BUTTON_WIDTH, BUTON_HEIGHT, "Add");
    let mut but_remove = Button::new(10 + 4 * (BUTTON_WIDTH + BUTON_SPACING), HEIGHT-50, BUTTON_WIDTH, BUTON_HEIGHT, "Remove");
    let mut but_save = Button::new(10 + 5 * (BUTTON_WIDTH + BUTON_SPACING), HEIGHT-50, BUTTON_WIDTH, BUTON_HEIGHT, "Save");
    let mut append_check = CheckButton::new(10 + 6 * (BUTTON_WIDTH + BUTON_SPACING), HEIGHT-50, 2* BUTTON_WIDTH, BUTON_HEIGHT, "add");
    let mut input_file = FileInput::new(10 + 7 * (BUTTON_WIDTH + BUTON_SPACING), HEIGHT-50, 2* BUTTON_WIDTH, BUTON_HEIGHT, "");
    let mut but_exit = Button::new(10 + 9 * (BUTTON_WIDTH + BUTON_SPACING), HEIGHT-50, BUTTON_WIDTH, BUTON_HEIGHT, "Exit");
    let bottom_buf = TextBuffer::default();
    let right_buf = TextBuffer::default();
    let mut bottom_disp = TextDisplay::default().with_pos(0, IM_HEIGHT+30).with_size(IM_WIDTH, 90);
    let top_disp = output::Output::default().with_pos(0, IM_HEIGHT).with_size(IM_WIDTH - 100, 30);
    let mut pos_disp = output::Output::default().with_pos(IM_WIDTH-100, IM_HEIGHT).with_size(100, 30);
    let mut right_disp =  TextDisplay::default().with_pos(IM_WIDTH, 0).with_size(300, IM_HEIGHT+30);
    
    frame.set_color(Color::Red);
   
    input_file.set_value("baza.csv");
    append_check.set_value(false);

    bottom_disp.set_color(Color::Light1);
    bottom_disp.set_text_color(Color::Black);
    bottom_disp.set_buffer(Some(bottom_buf));
    bottom_disp.set_insert_position(bottom_disp.buffer().unwrap().length());

    right_disp.set_color(Color::Dark1);
    right_disp.set_text_color(Color::Black);
    right_disp.set_buffer(Some(right_buf));
    right_disp.set_insert_position(bottom_disp.buffer().unwrap().length());
    
    let im: ImageAnnotation = ImageAnnotation { img: None, width: None, height: None, file_name: None, annotations: None };
    let mut imgs: Vec<ImageAnnotation> = Vec::new();
    imgs.push(im);
    let imgs2 = Rc::from(RefCell::from(imgs));
    let annotator: Annotator = Annotator { current_pos: Rc::from(RefCell::from(0)), current_img: Rc::from(RefCell::from(None)), current_coords: Rc::from(RefCell::from(None)), images: imgs2 };
    let mut image_paths = Vec::new();

    wind.end();
    wind.show();

    let surf = ImageSurface::new(frame.width(), frame.height(), false);
    ImageSurface::push_current(&surf);
    ImageSurface::pop_current();
    let surf = Rc::from(RefCell::from(surf));

    frame.draw({
        let surf = surf.clone();
        move |f| {
            let surf = surf.borrow();
            let mut img = surf.image().unwrap();
            img.draw(f.x(), f.y(), f.w(), f.h());
        }
    });

    frame.handle({
        wind.redraw();
        let mut annotator_clone = annotator.clone();
        let mut x = 0;
        let mut y = 0;
        let surf = surf.clone();
        move |f, ev| {
           
            let surf = surf.borrow_mut();
      
            match ev {
                Event::Move => {
                    let mouse_coords = app::event_coords();
                    let txt = format!("x={}, y={}", mouse_coords.0, mouse_coords.1);
                    pos_disp.set_value(&txt);

                    true
                }

                Event::Push => {
                    ImageSurface::push_current(&surf);
                    set_draw_color(Color::Red);
                    set_line_style(LineStyle::Solid, 1);
                    let coords = app::event_coords();
                    x = coords.0;
                    y = coords.1;
                    draw_rectf(x, y, 2, 2);
                    ImageSurface::pop_current();
                    f.redraw();
                    true
                }

                Event::Released =>
                {
                    ImageSurface::push_current(&surf);
                    set_draw_color(Color::Red);
                    set_line_style(LineStyle::Dot, 2);
                    
                    let coords = app::event_coords();
                    annotator_clone.update_current_coords(Coords{x:x, y:y, width:coords.0-x, height:coords.1-y});
                    
                    draw_rect(x, y, coords.0-x, coords.1-y);

                    ImageSurface::pop_current();
                    f.redraw();
            
                    true
                }

                Event::Drag => {
                    if !annotator_clone.get_current_img().is_none() {

                        ImageSurface::push_current(&surf);
                        set_draw_color(Color::Red);
                        set_line_style(LineStyle::Dot, 2);
                
                        let coords = app::event_coords();
                    

                        annotator_clone.get_current_img().unwrap().draw(f.x(), f.y(), f.w(), f.h());
                        
                        draw_rect(x, y, coords.0-x, coords.1-y);

                        ImageSurface::pop_current();
                        f.redraw();
                    }

                    true
                }
                _ => false,
            }
        }
    });

    but_select.set_callback({
        let mut annotator_clone = annotator.clone();
        let mut but_next_clone = but_next.clone();
        let mut input_file_clone = input_file.clone();
        let right_disp_clone = right_disp.clone();

        move |_| {

            let new_folder_path = FileDialog::new().set_directory(".").pick_folder();
            let source_path = new_folder_path.unwrap();
            image_paths.clear();

            for e in WalkDir::new(&source_path).into_iter().filter_map(|e| e.ok()) {
            
                let file_path = String::from(e.path().to_str().unwrap());
                let file_path_clone = file_path.clone();

                if file_path_clone.to_lowercase().ends_with(".png") {

                    image_paths.push(file_path_clone);
                    
                }
            }
            annotator_clone.load_images(image_paths.clone());

            right_disp_clone.buffer().unwrap().set_text("");
            for p in image_paths.clone(){
                let filename = Path::new(&p).file_name().unwrap();

                let txt = format!("{:?}\n\r", filename).replace("\"", "");

                right_disp_clone.buffer().unwrap().append(&txt);        
                

            }
            
            let full_path = source_path.to_str().unwrap();
        
            let p:Vec<&str>= full_path.split("\\").collect();
            let txt = p.last().unwrap();
            let txt =  txt.replace("\"", "");
        
            input_file_clone.set_value(&format!("{}_baza.csv", txt));
            but_next_clone.do_callback();
            
    }

    });

    but_next.set_callback({ 
        let mut top_disp = top_disp.clone();

        let mut f = frame.clone();
        let mut annotator_clone = annotator.clone();
        let surf = surf.clone();
        move |_| {    

            let surf = surf.borrow_mut();

            ImageSurface::push_current(&surf);
            annotator_clone.increment_pos();

            annotator_clone.update_current_img();
    
            let img = annotator_clone.get_current_img();
            if !img.is_none() {
                let mut img = img.unwrap();

                //img.scale(f.w(), f.h(), true, false);
                
                img.draw(f.x(), f.y(), f.w(), f.h());
                let co = annotator_clone.get_coords_by_pos();
                if !co.is_none(){
                    let co = co.unwrap();
                    set_draw_color(Color::Red);
                    set_line_style(LineStyle::Dot, 2);
                    draw_rect(co.x, co.y, co.width, co.height);

                }
                ImageSurface::pop_current();
                f.redraw();

                let txt = annotator_clone.get_current_img_info_text();
                top_disp.set_value(&txt);
                //&top_disp.buffer().unwrap().set_text(&txt);

            }   

        }
    });

    but_prev.set_callback({
        let mut f = frame.clone();
        let mut annotator_clone = annotator.clone();
        let surf = surf.clone();
        let mut top_disp = top_disp.clone();

        move |_| {   

            let surf = surf.borrow_mut();

            ImageSurface::push_current(&surf);
            annotator_clone.decrement_pos();

            annotator_clone.update_current_img();

            let img = annotator_clone.get_current_img();
            if !img.is_none() {
                let mut img = img.unwrap();

                //img.scale(f.w(), f.h(), true, false);

                img.draw(f.x(), f.y(), f.w(), f.h());
                let co = annotator_clone.get_coords_by_pos();
                if !co.is_none(){
                    let co = co.unwrap();
                    set_draw_color(Color::Red);
                    set_line_style(LineStyle::Dot, 2);
                    draw_rect(co.x, co.y, co.width, co.height);

                }
                ImageSurface::pop_current();
                f.redraw();
                
                let txt = format!("{}", annotator_clone.get_current_img_info_text());
                top_disp.set_value(&txt);
            }   
            
        }
        
    });

    

    but_save.set_callback({

        let annotator_clone = annotator.clone();
        let bottom_disp_clone = bottom_disp.clone();
        let right_disp_clone = right_disp.clone();

        move |_| {  

            let filename = input_file.value();
            annotator_clone.save_csv(&filename, append_check.value()) ; 

            let txt = format!("saved to {}\n\r",&filename );
            bottom_disp_clone.buffer().unwrap().append(&txt);

            let contents = read_to_string(&filename).unwrap();

            let txt = format!("{}", contents);

            right_disp_clone.buffer().unwrap().set_text(&txt);

            // for row in metadata_reader.records() {
            //     let r = row.unwrap().clone();
            //     right_disp_clone.buffer().unwrap().append(&format!("{:?}\n\r",&r));   
            // }


        }
        
    });

    but_add.set_callback({
        let mut annotator_clone = annotator.clone();
        let bottom_disp_clone = bottom_disp.clone(); 

        move |_| {  
            annotator_clone.add_coords();

            let txt = format!("{}\n\r", annotator_clone.get_current_img_info_text());
            bottom_disp_clone.buffer().unwrap().append(&txt);
        }
        
    });

    but_remove.set_callback({
        let mut annotator_clone = annotator.clone();
        let bottom_disp_clone = bottom_disp.clone(); 
        let mut f = frame.clone();

        move |_| {  
            annotator_clone.remove_coords();
            let txt = format!("removed {}\n\r", annotator_clone.get_current_img_info_text());
            bottom_disp_clone.buffer().unwrap().append(&txt);

            let surf = surf.borrow_mut();

            ImageSurface::push_current(&surf);
         
            annotator_clone.update_current_img();

            let img = annotator_clone.get_current_img();
            if !img.is_none() {
                let mut img = img.unwrap();

                img.draw(f.x(), f.y(), f.w(), f.h());
                let co = annotator_clone.get_coords_by_pos();
                if !co.is_none(){
                    let co = co.unwrap();
                    set_draw_color(Color::Red);
                    set_line_style(LineStyle::Dot, 2);
                    draw_rect(co.x, co.y, co.width, co.height);

                }
                ImageSurface::pop_current();
                f.redraw();
            }

        }
        
    });

    but_exit.set_callback({

        move |_| {  

            std::process::exit(0);     
        }
        
    });

    app.run().unwrap();

}

