

use fltk::{
    prelude::*, image::{PngImage}, 
};
use std::{cell::{RefCell}};
use std::rc::Rc;
use csv::Writer;
use std::path::Path;
use std::fs::OpenOptions;

use std::fmt;
use image::*;
use imageproc::drawing::draw_line_segment_mut;
use config_file::FromConfigFile;

use crate::config_for_annotation::{self, Config};



const IM_WIDTH: i32 = 700;
const IM_HEIGHT: i32 = 500;
const BORDER: i32 = 10;



#[derive(Clone, Debug)]
pub struct RecCoords{
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

#[derive(Clone, Debug)]
pub struct PointCoords{
    pub x: i32,
    pub y: i32,
}

impl fmt::Display for PointCoords {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({},{})", self.x, self.y)
    }
}

#[derive(Clone, Debug)]
pub struct USGImage{
    pub img: Option<PngImage>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub file_name: Option<String>,
    pub rec_annotation: Option<RecCoords>,
    pub point_annotation: Option<Vec<PointCoords>>,
    pub resize_ratio: f32,
   
}

impl USGImage {

    pub fn add_image(&mut self, _file_name: String) {
        
        self.file_name =  Some(_file_name.clone());   

        let mut img = PngImage::load(&_file_name).unwrap();
        let (base_width, base_height) = (img.width(), img. height());

        img.scale(IM_WIDTH, IM_HEIGHT, true, false);
        let (scaled_width, scaled_height) = (img.width(), img. height());

        self.width = Some(img.width());
        self.height = Some(img.height());
        self.img = Some(img);  
        self.resize_ratio = (scaled_width as f32)/(base_width as f32);
       
    }

}

#[derive(Clone, Debug)]
pub struct Annotator {
    pub current_pos: Rc<RefCell<usize>>,
    pub current_img:Rc<RefCell<Option<PngImage>>>,
    pub images: Rc<RefCell<Vec<USGImage>>>,
    pub config: Option<config_for_annotation::Config>
}



pub fn export_rect_image( _source:String, _dest: String, _rec:RecCoords, _ratio: f32) {

    let (w, h) = (((_rec.width as f32)/_ratio) as u32, ((_rec.height as f32)/_ratio) as u32);
    let (x, y) = (((_rec.x as f32)/_ratio) as u32, ((_rec.y as f32)/_ratio) as u32);

    let mut img = image::open(Path::new(&_source)).unwrap();
    let sub_img = imageops::crop(&mut img, x, y, w, h);
    sub_img
        .to_image()
        .save_with_format(&_dest, ImageFormat::Png)
        .unwrap();

}

pub fn export_points_image(_source:String, _dest: String, _points:Vec<PointCoords>, _ratio: f32) {

    let img = image::open(Path::new(&_source)).unwrap();
    let mut img = img.to_rgb8();

    let color_red =image::Rgb([255,0,0]);
    let mut prev_x = 0;
    let mut prev_y = 0;
    let mut i = 0;

    
    for po in _points{

        let (x, y) = (((po.x as f32)/_ratio) as u32, ((po.y as f32)/_ratio) as u32);


        if i==0 {
            img.put_pixel(x, y, color_red);
            i = 1;
            prev_x = x;
            prev_y = y;

        }else {

            draw_line_segment_mut(&mut img, (prev_x as f32, prev_y as f32), (x as f32, y as f32), color_red);
            prev_x = x;
            prev_y = y;

        }

    }

    img.save_with_format(&_dest, ImageFormat::Png).unwrap();


}



pub fn export_rec_from_points_image(_source:String, _dest: String, _points:Vec<PointCoords>, _ratio: f32) {

    let img = image::open(Path::new(&_source)).unwrap();
    let img = img.to_rgb8();

    let (width, height) = (img.width() as i32, img.height() as i32); 

    let mut min_x:i32 = 1000;
    let mut max_x:i32 = 0;
    let mut min_y:i32 = 1000;
    let mut max_y:i32 = 0;
    
    for po in _points{

        let (x, y) = (((po.x as f32)/_ratio) as u32, ((po.y as f32)/_ratio) as u32);

        if (x as i32) < min_x { min_x = x as i32; }
        if (x as i32) > max_x { max_x = x as i32; }
        if (y as i32) < min_y { min_y = y as i32; }
        if (y as i32) > max_y { max_y = y as i32; }


    }


    if (min_x - BORDER) > 0 {min_x = min_x - BORDER}
    if (max_x + BORDER) < width {max_x = max_x + BORDER}
    if (min_y - BORDER) > 0 {min_y = min_y - BORDER}
    if (max_y + BORDER) < height {max_y = max_y + BORDER}


    let co = RecCoords{x: (min_x as i32), y: (min_y as i32), width: (max_x-min_x) as i32, height: (max_y - min_y) as i32};

    let tmp = _dest.clone();

    let dest_filename = tmp.replace("out_shape", "out_rec_from_shape");

    export_rect_image(_source.clone(), dest_filename, co, 1.0);

}

impl Annotator {


    pub fn load_images(&mut self, _paths:  Vec<String>) {

        let mut tmp: Vec<USGImage> = Vec::new();
        for p in _paths
        {
            let mut a:USGImage = USGImage { img: None, width: (None), height: (None), file_name: (None), rec_annotation: (None), point_annotation: (None), resize_ratio:0.0 };
            a.add_image(p);
            tmp.push(a);
        }
        *self.images.borrow_mut() = tmp;   
    }

    pub fn get_current_img(&mut self) -> Option<PngImage>{

        let res = (*self.current_img.borrow_mut()).clone();
        
        return res;

    }

    pub fn is_empty(&mut self) -> bool{

        let res = (*self.current_img.borrow_mut()).clone().is_none();
        
        return res;

    }

    pub fn get_current_path(&mut self) -> Option<String>{

        let pos = *self.current_pos.borrow();
        let tmp = (*self.images.borrow_mut())[pos].clone();
        let res = tmp.file_name;

        return res;

    }


    pub fn get_current_rec_coords(&mut self) -> Option<RecCoords>{

        let pos = *self.current_pos.borrow();
        let tmp = (*self.images.borrow_mut())[pos].clone();
        let res = tmp.rec_annotation;

        return res;

    }

    pub fn update_current_img(&mut self) {

        let p =  *self.current_pos.borrow();
        

        let k = (*self.images.borrow().clone())[p].img.clone();
        
        *self.current_img.borrow_mut() = k;
 
    }


    pub fn add_rec_coords(&mut self, _coords:  RecCoords) {

        let p =  *self.current_pos.borrow();

        (*self.images.borrow_mut())[p].rec_annotation = Some(_coords);
        
    }

    pub fn add_point_coords(&mut self, point:PointCoords) {

        let p =  *self.current_pos.borrow();
        let p1 = (*self.images.borrow_mut()).clone();


        let p2 = p1[p].clone().point_annotation;

        if p2.is_none() {
            let mut tmp: Vec<PointCoords> = Vec::new();
            
            tmp.push(point);
            
            (*self.images.borrow_mut())[p].point_annotation = Some(tmp);

        } else {
       
            
            let mut p2 = p2.unwrap();
            p2.push(point);
            (*self.images.borrow_mut())[p].point_annotation = Some(p2);
        }
        
    }

    pub fn remove_last_point(&mut self) {

        let p =  *self.current_pos.borrow();
        let p1 = (*self.images.borrow_mut()).clone();


        let p2 = p1[p].clone().point_annotation;

        if !p2.is_none() {
            let mut p2 = p2.unwrap();
            let last_pos = p2.len()-1;
            p2.remove(last_pos);

            (*self.images.borrow_mut())[p].point_annotation = Some(p2);
        }
        
    }


    pub fn get_current_point_coords(&mut self) -> Option<Vec<PointCoords>> {

        let pos = *self.current_pos.borrow();
        let tmp = (*self.images.borrow_mut())[pos].clone();
        let res = tmp.point_annotation;

        return res;
    }



    pub fn remove_all_coords(&mut self) {

        let p =  *self.current_pos.borrow();

        (*self.images.borrow_mut())[p].rec_annotation = None;
        (*self.images.borrow_mut())[p].point_annotation = None;

    }

    pub fn increment_pos(&mut self) {

        let size = (*self.images.borrow().clone()).len();
        let pos = *self.current_pos.borrow();

        if pos < size - 1 { 
            *self.current_pos.borrow_mut() += 1;
        }

    }

    pub fn reset_pos(&mut self) {

         *self.current_pos.borrow_mut() = 0;

    }

    pub fn decrement_pos(&mut self) {

        let pos = *self.current_pos.borrow();

        if pos > 0  { 
            *self.current_pos.borrow_mut() -= 1;
        }
        
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

        if !curr_image.rec_annotation.is_none() {
            let im_coords = curr_image.rec_annotation.unwrap();
            let coords_txt = format!("| x={:?}, y={:?}, w={:?}, h={:?}",&im_coords.x, &im_coords.y, &im_coords.width, &im_coords.height);

            res = format!("{} {}", res, coords_txt);
         }
        return res;
    }

    pub fn load_csv(&self, _source: &str,) {

        let mut tmp: Vec<USGImage> = Vec::new();

        let mut metadata_reader = csv::Reader::from_path(&_source).unwrap();

        for f in metadata_reader.records() {

            let row = f.unwrap().clone();
            let file_name = String::from(&row[1]);

            let annotation_points = &row[8];
            
            if !(annotation_points == "") {

                let mut annotation_points = String::from(annotation_points);
                annotation_points.remove(0);
                annotation_points.remove(annotation_points.len()-1);
                

                let spl = annotation_points.split(")(");
                let mut points : Vec<PointCoords> = Vec::new();

                for s in spl{
                    let co :Vec<&str> = s.split(',').collect();
                    let x:i32 = co[0].parse().unwrap();
                    let y:i32 = co[1].parse().unwrap();

                    let co = PointCoords{x:x, y:y};
                    points.push(co);
                }

                let mut a:USGImage = USGImage { img: None, width: (None), height: (None), file_name: (None), rec_annotation: (None), point_annotation: Some(points), resize_ratio:0.0  };
                a.add_image(file_name);
                tmp.push(a);
            }

        }

        *self.images.borrow_mut() = tmp;  
    }

    pub fn reload_config(&mut self, _config_file: String){

        let cfg = Config::from_config_file(_config_file).unwrap();
        self.config = Some(cfg); 

    }


    pub fn save_csv(&self, _dest: &str, _append:bool) {

        let mut i = 0;
        let cfg = self.config.clone();
        let cfg = cfg.unwrap();


        let file = OpenOptions::new().create(true).write(true).truncate(true).append(false).open(&_dest).unwrap();
        let mut csv_writer_metadata = Writer::from_writer(file);
        csv_writer_metadata.write_record(&["lp", "filename", "img_width", "img_height", "annotation_x", "annotation_y", "annotation_w", "annotation_h", "annotation_points"]).ok();


        // if _append {
        //     let file = OpenOptions::new().write(true).append(true).open(&_dest).unwrap();
        //     let mut csv_writer_metadata = Writer::from_writer(file);
        
        // } else {
            

        //     let file = OpenOptions::new().create(true).write(true).truncate(true).append(false).open(&_dest).unwrap();
        //     let mut csv_writer_metadata = Writer::from_writer(file);
        //     csv_writer_metadata.write_record(&["lp", "filename", "img_width", "img_height", "annotation_x", "annotation_y", "annotation_w", "annotation_h", "annotation_points"]).ok();

        // }   


        let images = (*self.images.borrow_mut()).clone();
        let mut csv_row =  Vec::new();
        
        
        for im in images {
            i += 1;
            csv_row.clear();

            let im_rec_coords = im.rec_annotation.clone();
            let im_points = im.point_annotation.clone();

            if (!im_rec_coords.is_none()) || (!im_points.is_none()){

                let file_name = im.file_name.clone();
                csv_row.push(i.to_string());
                csv_row.push(file_name.unwrap()); 
                csv_row.push(im.width.unwrap().to_string()); 
                csv_row.push(im.height.unwrap().to_string());

                if !(im_rec_coords.is_none()) {
                    let im_rec_coords = im_rec_coords.unwrap();
                    csv_row.push(im_rec_coords.x.to_string()); 
                    csv_row.push(im_rec_coords.y.to_string()); 
                    csv_row.push(im_rec_coords.width.to_string()); 
                    csv_row.push(im_rec_coords.height.to_string());

                    
                    let source_filename = im.file_name.clone();
                    let source_filename = source_filename.unwrap();

                    let dest_file_name = Path::new(&source_filename).file_name().unwrap();
                    let dest_file_name = String::from(dest_file_name.to_string_lossy());
                    let dest_file_name = format!("{}/out_rect_{}", cfg.annotations_folder, dest_file_name);
                    

                    export_rect_image(source_filename, dest_file_name, im_rec_coords, im.resize_ratio);
                
                } else {
                    csv_row.push(String::from(""));
                    csv_row.push(String::from(""));
                    csv_row.push(String::from(""));
                    csv_row.push(String::from(""));

                }

                if !im_points.is_none(){
                    let im_points = im_points.unwrap();
                    let im_points_clone = im_points.clone();
                    let mut str_points = String::from("");
                    for p in im_points{

                        let tmp = format!("{}", p);
                        str_points.push_str(&tmp);

                    }
                    
                    let source_filename = im.file_name.clone();
                    let source_filename = source_filename.unwrap();

                    let dest_file_name = Path::new(&source_filename).file_name().unwrap();
                    let dest_file_name = String::from(dest_file_name.to_string_lossy());
                    let dest_file_name = format!("{}/out_shape_{}",cfg.annotations_folder, dest_file_name);

                    export_points_image(source_filename.clone(), dest_file_name.clone(), im_points_clone.clone(), im.resize_ratio);

                    let dest_file_name = Path::new(&source_filename).file_name().unwrap();
                    let dest_file_name = String::from(dest_file_name.to_string_lossy());
                    let dest_file_name = format!("{}/out_shape_{}",cfg.images_folder, dest_file_name);
                  
                    export_rec_from_points_image(source_filename, dest_file_name, im_points_clone, im.resize_ratio);

                    csv_row.push(str_points);

                } else {
                    csv_row.push(String::from(""));
                }

                csv_writer_metadata.write_record(&csv_row).ok();
                

            }         
  
        }

    }

}
