use fltk::{
    app::{self},button::{Button, RadioRoundButton}, output,
    draw::{set_draw_color, set_line_style, LineStyle, draw_rect, draw_point, draw_line},
    enums::{Color, Event},
    frame::Frame,
    prelude::*,text::TextBuffer,text::TextDisplay, input::FileInput, button::CheckButton, input::Input ,
    surface::ImageSurface,
    window::Window, 
};
use std::{cell::{RefCell}};
use std::rc::Rc;
use rfd::FileDialog;
use walkdir::WalkDir;
use std::path::Path;
use std::fs::read_to_string;
use image::{DynamicImage, imageops, GenericImage};
use std::env::current_dir;
use config_file::FromConfigFile;
use serde::Deserialize;
use std::fs;


pub mod config_for_annotation;
use config_for_annotation::*;


mod annotator;
use annotator::*;



const WIDTH: i32 = 1000;
const HEIGHT: i32 = 700;
const IM_WIDTH: i32 = 700;
const IM_HEIGHT: i32 = 500;
const BUTTON_WIDTH: i32 = 70;
const BUTON_HEIGHT: i32 = 30;
const BUTON_SPACING: i32 = 10;
const CONFIG_FILE:&str = "config.toml";

fn annotation_redraw(_annotator: Annotator, _surf:Rc<RefCell<ImageSurface>>, _frame: Frame){

    let mut annotator_clone = _annotator.clone();
    let mut f = _frame.clone();
    let surf = _surf.clone();


    let surf = surf.borrow_mut();

    ImageSurface::push_current(&surf);
 
    annotator_clone.update_current_img();

    let img = annotator_clone.get_current_img();

    if !img.is_none() {
        let mut img = img.unwrap();
        
        img.draw(f.x(), f.y(), f.w(), f.h());
        let co = annotator_clone.get_current_rec_coords();
        if !co.is_none(){
            let co = co.unwrap();
            set_draw_color(Color::Red);
            set_line_style(LineStyle::Solid, 2);
            draw_rect(co.x, co.y, co.width, co.height);

        }

        let po = annotator_clone.get_current_point_coords();
        if !po.is_none(){
            let po = po.unwrap();
            set_draw_color(Color::Red);
            let mut prev_x = 0;
            let mut prev_y = 0;
            let mut i = 0;

            for p in po{
                if i==0 {
                    draw_point(p.x, p.y);
                    i = 1;
                    prev_x = p.x;
                    prev_y = p.y;

                }else {

                    draw_line(prev_x, prev_y, p.x, p.y);
                    prev_x = p.x;
                    prev_y = p.y;

                }
                
                
            }
        }

        ImageSurface::pop_current();
        f.redraw();
    }
}


fn detect_edges(_rgb_image: String, _coords: RecCoords){
    let total_start = std::time::Instant::now();

    let source_image = image::open(_rgb_image)
        .expect("failed to read image")
        .to_luma8();

    let source_image =source_image.clone();
    let source_image =  DynamicImage::from(source_image);
    let mut source_image = source_image.resize(IM_WIDTH as u32, IM_HEIGHT as u32 , imageops::FilterType::Nearest);
   
    let source_image = source_image.sub_image(_coords.x as u32, _coords.y as u32, _coords.width as u32, _coords.height as u32);
    let source_image = source_image.to_image();
    let source_image =  DynamicImage::from(source_image);
    let source_image = source_image.to_luma8();

    //let source_image = source_image.to_luma8();
    println!("total time: {:?} seconds", total_start.elapsed().as_secs_f32());
    // let r = _rgb_image.to_rgb_image().unwrap();

    // let tmp = _rgb_image.to_rgb_data();

    // let w = _rgb_image.width() as u32;
    // let h = _rgb_image.height() as u32;

    // println!("w={}, h={}", w, h);


    // let im1= ImageBuffer::from_raw(w, h, *tmp).unwrap();
    // let im2:image::RgbImage = image::RgbImage::from_raw(w, h, tmp).unwrap();
    // let source_image = DynamicImage::ImageRgb8(im2);

    // source_image.save("C:/datasets/COI/v2/baza/png/1/_2.png");

    // let source_image2 = source_image.to_luma8();


    
    //let source_image2 = ImageBuffer::from_raw(200,200, _rgb_image.to_rgb_data());

    // println!("total time: {:?} seconds", total_start.elapsed().as_secs_f32());
    // let detection = edge_detection::canny(
    //     source_image,
    //     1.2,  // sigma
    //     0.3,  // strong threshold
    //     0.1, // weak threshold
    // );


    // println!("total time: {:?} seconds", total_start.elapsed().as_secs_f32());

    // let tmp = detection.as_image();
    // let res = format!("out_edges.png");
    // tmp.save(&res);
    
    println!("total time: {:?} seconds", total_start.elapsed().as_secs_f32());
}


fn main() {
  
    let app = app::App::default().with_scheme(app::Scheme::Gtk);
    let mut wind = Window::new(100, 50, WIDTH, HEIGHT, "Image annotator");
    let mut frame = Frame::default().with_size(IM_WIDTH, IM_HEIGHT).with_pos(0, 0);
    
    let mut but_select = Button::new(10, HEIGHT - 45, BUTTON_WIDTH, BUTON_HEIGHT, "Select dir");
    let mut but_prev = Button::new(10 + 1 * (BUTTON_WIDTH + BUTON_SPACING), HEIGHT-45, BUTTON_WIDTH, BUTON_HEIGHT, "Prev");
    let mut but_next = Button::new(10 + 2 * (BUTTON_WIDTH + BUTON_SPACING), HEIGHT-45, BUTTON_WIDTH, BUTON_HEIGHT, "Next");
    let mut but_remove = Button::new(10 + 3 * (BUTTON_WIDTH + BUTON_SPACING), HEIGHT-45, BUTTON_WIDTH, BUTON_HEIGHT, "Remove");
    let mut but_save = Button::new(10 + 4 * (BUTTON_WIDTH + BUTON_SPACING), HEIGHT-45, BUTTON_WIDTH, BUTON_HEIGHT, "Save");
    let mut but_load = Button::new(10 + 5 * (BUTTON_WIDTH + BUTON_SPACING), HEIGHT-45, BUTTON_WIDTH, BUTON_HEIGHT, "Load");
    let mut but_detect = Button::new(10 + 8 * (BUTTON_WIDTH + BUTON_SPACING), HEIGHT-45, BUTTON_WIDTH, BUTON_HEIGHT, "Detect");
    let radio_rec = RadioRoundButton::new(10 + 9 * (BUTTON_WIDTH + BUTON_SPACING), HEIGHT-50, BUTTON_WIDTH, BUTON_HEIGHT, "Rectangle");
    let mut radio_point = RadioRoundButton::new(10 + 9 * (BUTTON_WIDTH + BUTON_SPACING), HEIGHT-30, BUTTON_WIDTH, BUTON_HEIGHT, "Point");
    let mut but_exit = Button::new(10 + 11 * (BUTTON_WIDTH + BUTON_SPACING), HEIGHT-45, BUTTON_WIDTH, BUTON_HEIGHT, "Exit");

    let bottom_buf = TextBuffer::default();
    let right_buf = TextBuffer::default();



    let mut bottom_disp = TextDisplay::default().with_pos(0, IM_HEIGHT+60).with_size(IM_WIDTH, 90);
    let status_disp = output::Output::default().with_pos(0, IM_HEIGHT).with_size(IM_WIDTH - 100, 30);
    let mut position_disp = output::Output::default().with_pos(IM_WIDTH-100, IM_HEIGHT).with_size(100, 30);
    let mut input_get_csv_file = Input::new(0, IM_HEIGHT+30, IM_WIDTH, 30, "");
    let mut right_disp =  TextDisplay::default().with_pos(IM_WIDTH, 0).with_size(300, IM_HEIGHT+30);
    

    frame.set_color(Color::Red);
   
    input_get_csv_file.set_value("");

    bottom_disp.set_color(Color::Light1);
    bottom_disp.set_text_color(Color::Black);
    bottom_disp.set_buffer(Some(bottom_buf));
    bottom_disp.set_insert_position(bottom_disp.buffer().unwrap().length());

    radio_point.set_value(true);


    right_disp.set_color(Color::Dark1);
    right_disp.set_text_color(Color::Black);
    right_disp.set_buffer(Some(right_buf));
    right_disp.set_insert_position(bottom_disp.buffer().unwrap().length());
    
    
    let cfg = Config::from_config_file(CONFIG_FILE).unwrap();
    fs::create_dir_all(&cfg.images_folder);
    fs::create_dir_all(&cfg.annotations_folder);

    let im: USGImage = USGImage { img: None, width: None, height: None, file_name: None, rec_annotation: None, point_annotation: (None), resize_ratio: 0.0  };
    let mut imgs: Vec<USGImage> = Vec::new();
    imgs.push(im);
    let imgs2 = Rc::from(RefCell::from(imgs));
    let annotator: Annotator = Annotator { current_pos: Rc::from(RefCell::from(0)), current_img: Rc::from(RefCell::from(None)), images: imgs2, config: Some(cfg) };



    let config_contents = fs::read_to_string(CONFIG_FILE).unwrap();
    right_disp.buffer().unwrap().set_text(&String::from(config_contents));
    


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
        let mut prev_x = 0;
        let mut prev_y = 0;
        let surf = surf.clone();
        move |f, ev| {
           
            let surf = surf.borrow_mut();
      
            match ev {
                Event::Move => {
                    let mouse_coords = app::event_coords();
                    let txt = format!("x={}, y={}", mouse_coords.0, mouse_coords.1);
                    position_disp.set_value(&txt);

                    true
                }

                
                Event::Push => {
                    let button_clicked = app::event_button();
                    if button_clicked == 1 {
                        ImageSurface::push_current(&surf);

                        let coords = app::event_coords();
                        x = coords.0;
                        y = coords.1;
                        prev_x = coords.0;
                        prev_y = coords.1;
                        set_line_style(LineStyle::Solid, 2);
                        set_draw_color(Color::Red);
                        draw_point(x, y);
                        ImageSurface::pop_current();
                        f.redraw();
                    }

                    if button_clicked == 3 {
                        ImageSurface::push_current(&surf);
                        annotator_clone.remove_last_point();
                        ImageSurface::pop_current();
                        f.redraw();

                    }


                    true
                }

                Event::Released =>
                {
                    ImageSurface::push_current(&surf);
                    
                    if radio_rec.value() {

                        let coords = app::event_coords();
                        annotator_clone.add_rec_coords(RecCoords{x:x, y:y, width:coords.0-x, height:coords.1-y});
                        set_line_style(LineStyle::Dot, 2);
                        set_draw_color(Color::Red);
                        draw_rect(x, y, coords.0-x, coords.1-y);
                        
                    }
                    ImageSurface::pop_current();
                    f.redraw();
            
                    true
                }

                Event::Drag => {
                    if !annotator_clone.get_current_img().is_none() {

                        ImageSurface::push_current(&surf);
                        
                        set_draw_color(Color::Red);
                        set_line_style(LineStyle::Solid, 2);

                        let coords = app::event_coords();

                        if radio_rec.value() {
                            annotator_clone.get_current_img().unwrap().draw(f.x(), f.y(), f.w(), f.h());
                            draw_rect(x, y, coords.0-x, coords.1-y);
                        } else{
                            draw_line(prev_x, prev_y, coords.0, coords.1);
                            //draw_point(coords.0, coords.1);
                            let p = PointCoords { x: coords.0, y: coords.1};
                            annotator_clone.add_point_coords(p);

                            prev_x = coords.0;
                            prev_y = coords.1;
                        }

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
        let mut input_file_clone = input_get_csv_file.clone();
        let bottom_disp_clone = bottom_disp.clone();

        move |_| {

            let new_folder_path = FileDialog::new().set_directory(".").pick_folder();

            if !new_folder_path.is_none(){

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
                annotator_clone.reset_pos();
                annotator_clone.update_current_img();

                but_next_clone.do_callback();

                let full_path = source_path.to_str().unwrap();            
                let p:Vec<&str>= full_path.split("\\").collect();
                let txt = p.last().unwrap();
                let txt =  txt.replace("\"", "");
            
                let curr_path = current_dir().unwrap();
                let curr_path = String::from(curr_path.to_string_lossy());

                input_file_clone.set_value(&format!("{}/{}_baza.csv", curr_path, txt));
                bottom_disp_clone.buffer().unwrap().set_text("");
                let txt = format!("loaded csv: {}\n\r", &full_path);
                bottom_disp_clone.buffer().unwrap().append(&txt);


            }
            
        }

    });

    but_next.set_callback({ 
        let mut status_disp = status_disp.clone();
        let  f = frame.clone();
        let mut annotator_clone = annotator.clone();
        let surf = surf.clone();
        move |_| {    

            if !annotator_clone.is_empty() {
                annotator_clone.increment_pos();
                annotation_redraw(annotator_clone.clone(), surf.clone(), f.clone());

                let txt = format!("{}", annotator_clone.get_current_img_info_text());
                status_disp.set_value(&txt);

            }
        }
    });

    but_prev.set_callback({
        let f = frame.clone();
        let mut annotator_clone = annotator.clone();
        let surf = surf.clone();
        let mut status_disp = status_disp.clone();

        move |_| {   
            if !annotator_clone.is_empty() {
                annotator_clone.decrement_pos();
                annotation_redraw(annotator_clone.clone(), surf.clone(), f.clone());

                let txt = format!("{}", annotator_clone.get_current_img_info_text());
                status_disp.set_value(&txt);
            }

            
        }
        
    });

    
    but_load.set_callback({
        let mut annotator_clone = annotator.clone();
        let mut but_next_clone = but_next.clone();
        let mut input_get_csv_file = input_get_csv_file.clone();
        let bottom_disp_clone = bottom_disp.clone();

        move |_| {

            let new_folder_path = FileDialog::new().set_directory(".").pick_file();

            if !new_folder_path.is_none(){
                let source_path = new_folder_path.unwrap();
                let source_path = String::from(source_path.to_string_lossy());
                
                annotator_clone.load_csv(&source_path);              
                
                annotator_clone.reset_pos();
                annotator_clone.update_current_img();
                but_next_clone.do_callback();

                bottom_disp_clone.buffer().unwrap().set_text("");
                let txt = format!("loaded csv: {}\n\r", &source_path);
                bottom_disp_clone.buffer().unwrap().append(&txt);
                
                let p:Vec<&str>= source_path.split("\\").collect();
                let txt = p.last().unwrap();
                let p:Vec<&str>= txt.split("_").collect();

                let txt =  p.first().unwrap();

                let curr_path = current_dir().unwrap();
                let curr_path = String::from(curr_path.to_string_lossy());

                input_get_csv_file.set_value(&format!("{}/{}_baza.csv",curr_path, txt));


                
            }

        }

    });


    but_save.set_callback({

        let mut annotator_clone = annotator.clone();
        let bottom_disp_clone = bottom_disp.clone();

        move |_| {  

            if !annotator_clone.is_empty() {
                let filename = input_get_csv_file.value();
                annotator_clone.save_csv(&filename, false) ; 

                let txt = format!("saved to {}\n\r",&filename );
                bottom_disp_clone.buffer().unwrap().append(&txt);

            }

        }
        
    });


    but_remove.set_callback({
        let mut annotator_clone = annotator.clone();
        let f = frame.clone();
        let surf = surf.clone();

        let bottom_disp_clone = bottom_disp.clone(); 

        move |_| {  
            
            if !annotator_clone.is_empty() {
                annotator_clone.remove_all_coords();

                let txt = format!("removed {}\n\r", annotator_clone.get_current_img_info_text());
                bottom_disp_clone.buffer().unwrap().append(&txt);

                annotation_redraw(annotator_clone.clone(), surf.clone(), f.clone());
            }

        }
      
    });

    but_exit.set_callback({

        move |_| {  

            std::process::exit(0);     
        }
        
    });

    but_detect.set_callback({
        
        let mut f = frame.clone();
        let mut annotator_clone = annotator.clone();
        let surf = surf.clone();
      
        move |_| { 

            if !annotator_clone.is_empty() {


                let surf = surf.borrow_mut();
                ImageSurface::push_current(&surf);
                
                let img_file = annotator_clone.get_current_path();
                let coords = annotator_clone.get_current_rec_coords();
                
                if !img_file.is_none() {
                    let img_file = img_file.unwrap();
                    let coords = coords.unwrap();
            
                    detect_edges(img_file, coords);

                    ImageSurface::pop_current();
                    f.redraw(); 

                }

            }

        }

    });

    app.run().unwrap();

}

