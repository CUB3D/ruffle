//! A skia based rendering backend for ruffle
//! //TODO: Supports both OpenGL and software rendering
//!


use ruffle_core::backend::render::{RenderBackend, BitmapSource, ShapeHandle, BitmapInfo, BitmapHandle, Transform, Bitmap, NullBitmapSource, JpegTagFormat};
use ruffle_core::shape_utils::{DistilledShape, DrawPath, DrawCommand};
use ruffle_core::backend::render::swf::{Glyph, DefineBitsLossless, FillStyle, LineCapStyle, LineJoinStyle};
use ruffle_core::Color;
use ruffle_core::matrix::Matrix;
use x11::xlib::{Display, GC};
use std::os::raw::{c_ulong, c_int, c_char};
use std::ffi::CString;
use std::cmp::min_by;
use std::collections::{BTreeSet, BTreeMap};
use skia_safe::{Image, Surface, Paint, Path, ColorFilter, ColorMatrix, ImageFilter, Shader, PathEffect};
use ruffle_core::swf::LineStyle;
use skia_safe::canvas::PointMode;
use skia_safe::paint::{Style, Cap, Join};
use blit::RawBlit;

type Error = Box<dyn std::error::Error>;


pub struct SkiaRenderBackend {
    blit: RawBlit,
    surface: skia_safe::Surface,
    bitmaps: Vec<skia_safe::Image>,
    shapes: Vec<skia_safe::Image>,

    width: i32,
    height: i32,
}

impl SkiaRenderBackend {
    pub fn new<W: raw_window_handle::HasRawWindowHandle, T: 'static>(window: &W, event_loop: &winit::event_loop::EventLoop<T>) -> Self {

        let width = 600;
        let height = 600;

        let mut surface = skia_safe::Surface::new_raster_n32_premul((width, height)).unwrap();
        let can = surface.canvas();
        can.clear(skia_safe::Color::WHITE);

        let blit = RawBlit::from_raw_window_handle(window);
        Self {
            blit,
            surface,
            width,
            height,
            bitmaps: Default::default(),
            shapes: Default::default(),
        }



        //         let screen = unsafe { x11::xlib::XDefaultScreen(xlib.display as *mut Display)};
        //
        //         let visual = unsafe { x11::xlib::XDefaultVisual(xlib.display as *mut Display, screen)};
        //
        //         let mut vinfo: x11::xlib::XVisualInfo = unsafe { std::mem::zeroed() };
        //         unsafe { x11::xlib::XMatchVisualInfo(xlib.display as *mut Display, screen, 32, x11::xlib::TrueColor, &mut vinfo as *mut _)};
        //         println!("Got vinfo {:?}", vinfo);
        //
        //         // let w = 100u32;
        //         // let h = 100u32;
        //         // let mut data = vec![0xFFu8 as i8; (w * 4 * h) as usize];
        //         // let img = unsafe { x11::xlib::XCreateImage(xlib.display as *mut Display, visual, 32, x11::xlib::ZPixmap, 0, (&mut data[..]).as_mut_ptr(), w, h, 32, (w as i32) * 4)};
        //         // println!("Got image {:x}", img as usize);
        //
        //         // println!("Got xlib {:?}", xlib);
        //
        //         // unsafe { x11::xlib::XPutImage(xlib.display as *mut Display, xlib.window, gc, img, 0, 0, 0, 0, h, w) };
        //
        //         // unsafe { x11::xlib::XFlush(xlib.display as *mut Display)};
    }
}

impl RenderBackend for SkiaRenderBackend {
    fn set_viewport_dimensions(&mut self, width: u32, height: u32) {
        // self.surface = skia_safe::surface::Surface::new_raster_n32_premul((width as i32, height as i32)).unwrap();
        // self.width = width as i32;
        // self.height = height as i32;
        //
        // self.win = minifb::Window::new("test", width as usize, height as usize, minifb::WindowOptions::default()).unwrap();

    }

    fn register_shape(&mut self, shape: DistilledShape, bitmap_source: &dyn BitmapSource) -> ShapeHandle {
        let (_width, _height) = (
            f32::max(
                (shape.shape_bounds.x_max - shape.shape_bounds.x_min).to_pixels() as f32,
                20.0,
            ),
            f32::max(
                (shape.shape_bounds.y_max - shape.shape_bounds.y_min).to_pixels() as f32,
                20.0,
            ),
        );

        // let (_width, _height) = (self.width, self.height);

        let mut shape_surface = Surface::new_raster_n32_premul((self.width, self.height));
        if let Some(mut shape_surface) = shape_surface {
            let can = shape_surface.canvas();
            can.translate((_width, _height));
            //can.translate((_width/2., _height/2.));
            // let mut m = skia_safe::Matrix::new_identity();
            // m.set_scale_x(1./20.);
            // m.set_scale_y(1./20.);
            // can.set_matrix(&m);

            for path in shape.paths {
                match path {
                    DrawPath::Stroke { commands, is_closed, style } => {
                        let mut path = Path::new();

                        let c = &style.color;
                        let col = skia_safe::Color::from_argb(c.a, c.r, c.g, c.b);
                        let mut paint = Paint::default();
                        paint.set_color(col);
                        paint.set_stroke_width(style.width.to_pixels() as f32);
                        paint.set_style(Style::Stroke);

                        paint.set_stroke_cap(match style.start_cap {
                            LineCapStyle::Round => Cap::Round,
                            LineCapStyle::None => Cap::default(),
                            LineCapStyle::Square => Cap::Square
                        });
                        match style.join_style {
                            LineJoinStyle::Round => {
                                paint.set_stroke_join(Join::Round);
                            },
                            LineJoinStyle::Bevel => {
                                paint.set_stroke_join(Join::Bevel);
                            }
                            LineJoinStyle::Miter(m) => {
                                paint.set_stroke_join(Join::Miter);
                                paint.set_stroke_miter(m.to_f32());
                            }
                        };

                        for com in commands {
                            match com {
                                DrawCommand::MoveTo { x, y } => {
                                    path.move_to((x.to_pixels() as f32, y.to_pixels() as f32));
                                }
                                DrawCommand::LineTo { x, y } => {
                                    path.line_to((x.to_pixels() as f32, y.to_pixels() as f32));
                                }
                                DrawCommand::CurveTo { x1, x2, y1, y2 } => {
                                    path.quad_to((x1.to_pixels() as f32, y1.to_pixels() as f32), (x2.to_pixels() as f32, y2.to_pixels() as f32));
                                }
                            }
                        }

                        if is_closed {
                            path.close();
                        }

                        can.draw_path(&path, &paint);
                    }
                    DrawPath::Fill { style, commands } => {
                        let mut paint = Paint::default();
                        match style {
                            FillStyle::Color(c) => {
                                paint.set_color(skia_safe::Color::from_argb(c.a, c.r, c.g, c.b));
                            }
                            FillStyle::Bitmap { id, is_repeating, is_smoothed, matrix } => {
                                // if let Some(bm) = self.bitmaps.get(*id as usize) {
                                //     paint.set_image_filter(ImageFilter::from_image(bm));
                                // } else {
                                    paint.set_color(skia_safe::Color::GREEN);
                                // }
                            },
                            FillStyle::FocalGradient { focal_point, gradient } => {
                                paint.set_color(skia_safe::Color::BLUE);
                            },
                            FillStyle::LinearGradient(g) => {
                                // paint.set_shader(Some(Shader::linear_gradient()))
                                paint.set_color(skia_safe::Color::YELLOW);
                            },
                            FillStyle::RadialGradient(g) => {
                                // paint.set_shader(Some(Shader::radial_gradient()))
                                paint.set_color(skia_safe::Color::MAGENTA);
                            }
                        };

                        paint.set_stroke_width(1.0);
                        paint.set_style(Style::Fill);

                        let mut path = Path::new();

                        for com in commands {
                            match com {
                                DrawCommand::MoveTo { x, y } => {
                                    path.move_to((x.to_pixels() as f32, y.to_pixels() as f32));
                                }
                                DrawCommand::LineTo { x, y } => {
                                    path.line_to((x.to_pixels() as f32, y.to_pixels() as f32));
                                }
                                DrawCommand::CurveTo { x1, x2, y1, y2 } => {
                                    path.quad_to((x1.to_pixels() as f32, y1.to_pixels() as f32), (x2.to_pixels() as f32, y2.to_pixels() as f32));
                                }
                            }
                        }

                        can.draw_path(&path, &paint);
                    }
                }
            }

            let shapeimg = shape_surface.image_snapshot();

            let handle = self.shapes.len();

            let png = shapeimg.encode_to_data(skia_safe::EncodedImageFormat::PNG).unwrap();
            std::fs::write(format!("./shape_{}_{}.png", shape.id, handle), png.as_bytes());

            self.shapes.push(shapeimg);
            ShapeHandle(handle)
        } else {
            ShapeHandle(0)
        }
    }

    fn replace_shape(&mut self, shape: DistilledShape, bitmap_source: &dyn BitmapSource, handle: ShapeHandle) {
        let shape = self.register_shape(shape, bitmap_source);

        let new_shape = self.shapes.swap_remove(shape.0);
        self.shapes[handle.0] = new_shape;
    }

    fn register_glyph_shape(&mut self, glyph: &Glyph) -> ShapeHandle {
        let shape = ruffle_core::shape_utils::swf_glyph_to_shape(glyph);
        let shape = ruffle_core::shape_utils::DistilledShape::from(&shape);
        let (_width, _height) = (
            f32::max(
                (shape.shape_bounds.x_max - shape.shape_bounds.x_min).to_pixels() as f32,
                20.0,
            ),
            f32::max(
                (shape.shape_bounds.y_max - shape.shape_bounds.y_min).to_pixels() as f32,
                20.0,
            ),
        );

        let mut shape_surface = Surface::new_raster_n32_premul((self.width, self.height));
        if let Some(mut shape_surface) = shape_surface {
            let can = shape_surface.canvas();
            // can.translate((0., (glyph.bounds.as_ref().unwrap().y_max.to_pixels() as f32 - glyph.bounds.as_ref().unwrap().y_min.to_pixels() as f32)));
            can.translate((_width, _height));
            // let mut m = skia_safe::Matrix::new_identity();
            // m.set_scale_x(1./20.);
            // m.set_scale_y(1./20.);
            // can.set_matrix(&m);

            for path in shape.paths {
                match path {
                    DrawPath::Stroke { commands, is_closed, style } => {
                        let mut path = Path::new();

                        let c = &style.color;
                        let col = skia_safe::Color::from_argb(c.a, c.r, c.g, c.b);
                        let mut paint = Paint::default();
                        paint.set_color(col);
                        paint.set_stroke_width(style.width.to_pixels() as f32);
                        paint.set_style(Style::Stroke);

                        paint.set_stroke_cap(match style.start_cap {
                            LineCapStyle::Round => Cap::Round,
                            LineCapStyle::None => Cap::default(),
                            LineCapStyle::Square => Cap::Square
                        });
                        match style.join_style {
                            LineJoinStyle::Round => {
                                paint.set_stroke_join(Join::Round);
                            },
                            LineJoinStyle::Bevel => {
                                paint.set_stroke_join(Join::Bevel);
                            }
                            LineJoinStyle::Miter(m) => {
                                paint.set_stroke_join(Join::Miter);
                                paint.set_stroke_miter(m.to_f32());
                            }
                        };

                        for com in commands {
                            match com {
                                DrawCommand::MoveTo { x, y } => {
                                    path.move_to((x.to_pixels() as f32, y.to_pixels() as f32));
                                }
                                DrawCommand::LineTo { x, y } => {
                                    path.line_to((x.to_pixels() as f32, y.to_pixels() as f32));
                                }
                                DrawCommand::CurveTo { x1, x2, y1, y2 } => {
                                    path.quad_to((x1.to_pixels() as f32, y1.to_pixels() as f32), (x2.to_pixels() as f32, y2.to_pixels() as f32));
                                }
                            }
                        }

                        if is_closed {
                            path.close();
                        }

                        can.draw_path(&path, &paint);
                    }
                    DrawPath::Fill { style, commands } => {
                        let col = match style {
                            FillStyle::Color(c) => {
                                skia_safe::Color::from_argb(c.a, c.r, c.g, c.b)
                            }
                            FillStyle::Bitmap { .. } => skia_safe::Color::GREEN,
                            FillStyle::FocalGradient { .. } => skia_safe::Color::BLUE,
                            FillStyle::LinearGradient { .. } => skia_safe::Color::YELLOW,
                            FillStyle::RadialGradient { .. } => skia_safe::Color::MAGENTA,
                        };
                        let mut paint = Paint::default();
                        paint.set_color(col);
                        paint.set_stroke_width(1.0);
                        paint.set_style(Style::Fill);

                        let mut path = Path::new();

                        for com in commands {
                            match com {
                                DrawCommand::MoveTo { x, y } => {
                                    path.move_to((x.to_pixels() as f32, y.to_pixels() as f32));
                                }
                                DrawCommand::LineTo { x, y } => {
                                    path.line_to((x.to_pixels() as f32, y.to_pixels() as f32));
                                }
                                DrawCommand::CurveTo { x1, x2, y1, y2 } => {
                                    path.quad_to((x1.to_pixels() as f32, y1.to_pixels() as f32), (x2.to_pixels() as f32, y2.to_pixels() as f32));
                                }
                            }
                        }

                        can.draw_path(&path, &paint);
                    }
                }
            }

            let shapeimg = shape_surface.image_snapshot();

            let handle = self.shapes.len();

            let png = shapeimg.encode_to_data(skia_safe::EncodedImageFormat::PNG).unwrap();
            std::fs::write(format!("./shape_{}_{}.png", shape.id, handle), png.as_bytes());

            self.shapes.push(shapeimg);
            ShapeHandle(handle)
        } else {
            ShapeHandle(0)
        }
    }

    fn register_bitmap_jpeg(&mut self, data: &[u8], jpeg_tables: Option<&[u8]>) -> Result<BitmapInfo, Error> {
        let data = ruffle_core::backend::render::glue_tables_to_jpeg(data, jpeg_tables);
        self.register_bitmap_jpeg_2(&data[..])
    }

    fn register_bitmap_jpeg_2(&mut self, data: &[u8]) -> Result<BitmapInfo, Error> {
        let bitmap = ruffle_core::backend::render::decode_define_bits_jpeg(data, None)?;

        let row_bytes = bitmap.width * 4;
        let info = skia_safe::ImageInfo::new_n32((bitmap.width as i32, bitmap.height as i32), skia_safe::AlphaType::Unpremul, None);
        let data = match bitmap.data {
            ruffle_core::backend::render::BitmapFormat::Rgba(x) => x,
            ruffle_core::backend::render::BitmapFormat::Rgb(x) => {
                let mut z = Vec::new();
                for y in x.chunks(3) {
                    z.push(0);
                    z.extend_from_slice(y);
                }
                z
            },
        };
        let data = skia_safe::Data::new_copy(&data);
        let image = skia_safe::Image::from_raster_data(&info, data, row_bytes as usize);
        println!("GOt image {:?}", image.is_some());

        let handle = self.bitmaps.len();
        self.bitmaps.push(image.unwrap());

        Ok(BitmapInfo {
            handle: BitmapHandle(handle),
            width: bitmap.width as u16,
            height: bitmap.height as u16,
        })
    }

    fn register_bitmap_jpeg_3(&mut self, jpeg_data: &[u8], alpha_data: &[u8]) -> Result<BitmapInfo, Error> {
        let bitmap =
            ruffle_core::backend::render::decode_define_bits_jpeg(jpeg_data, Some(alpha_data))?;

        let row_bytes = bitmap.width * 4;
        let info = skia_safe::ImageInfo::new_n32((bitmap.width as i32, bitmap.height as i32), skia_safe::AlphaType::Unpremul, None);
        let data = match bitmap.data {
            ruffle_core::backend::render::BitmapFormat::Rgba(x) => x,
            _ => panic!()
        };
        let data = skia_safe::Data::new_copy(&data);
        let image = skia_safe::Image::from_raster_data(&info, data, row_bytes as usize);
        println!("GOt image {:?}", image.is_some());

        let handle = self.bitmaps.len();
        self.bitmaps.push(image.unwrap());

        Ok(BitmapInfo {
            handle: BitmapHandle(handle),
            width: bitmap.width as u16,
            height: bitmap.height as u16,
        })
    }

    fn register_bitmap_png(&mut self, swf_tag: &DefineBitsLossless) -> Result<BitmapInfo, Error> {
        let bitmap = ruffle_core::backend::render::decode_define_bits_lossless(swf_tag)?;
        let row_bytes = bitmap.width * 4;
        let info = skia_safe::ImageInfo::new_n32((bitmap.width as i32, bitmap.height as i32), skia_safe::AlphaType::Unpremul, None);
        let data = match bitmap.data {
            ruffle_core::backend::render::BitmapFormat::Rgba(x) => x,
            _ => panic!()
        };
        let data = skia_safe::Data::new_copy(&data);
        let image = skia_safe::Image::from_raster_data(&info, data, row_bytes as usize);

        let handle = self.bitmaps.len();
        self.bitmaps.push(image.unwrap());

        Ok(BitmapInfo {
            handle: BitmapHandle(handle),
            width: bitmap.width as u16,
            height: bitmap.height as u16,
        })
    }

    fn begin_frame(&mut self, clear: Color) {
        let can = self.surface.canvas();
        let col = skia_safe::Color::from_argb(clear.a, clear.r, clear.g, clear.b);
        can.clear(col);

        // let mut p = Paint::default();
        // p.set_color(skia_safe::Color::RED);
        // can.draw_rect(skia_safe::Rect::new(0., 0., 50., 50.), &p);
    }

    fn render_bitmap(&mut self, bitmap: BitmapHandle, transform: &Transform, smoothing: bool) {
        if let Some(bm) = self.bitmaps.get(bitmap.0) {
            let can = self.surface.canvas();

            let mut mat = skia_safe::Matrix::new_identity();
            let matrix = transform.matrix;
            let (tx, ty) = (transform.matrix.tx.to_pixels() as f32, transform.matrix.ty.to_pixels() as f32);

            mat.set_translate_x(tx);
            mat.set_translate_y(ty);
            mat.set_skew_x(matrix.c);
            mat.set_skew_y(matrix.b);
            mat.set_scale_y(matrix.d);
            mat.set_scale_x(matrix.a);
            can.set_matrix(&mat);

            can.draw_image(bm, (0, 0), None);

            can.reset_matrix();
        }
    }

    fn render_shape(&mut self, shape: ShapeHandle, transform: &Transform) {
        if let Some(bm) = self.shapes.get(shape.0) {
            let can = self.surface.canvas();

            let (tx, ty) = (transform.matrix.tx.to_pixels() as f32, transform.matrix.ty.to_pixels() as f32);

            let mut mat = skia_safe::Matrix::new_identity();
            let matrix = transform.matrix;
            mat.set_translate_x(tx);
            mat.set_translate_y(ty);
            mat.set_skew_x(matrix.c);
            mat.set_skew_y(matrix.b);
            mat.set_scale_y(matrix.d);
            mat.set_scale_x(matrix.a);
            can.set_matrix(&mat);

            let mut paint = Paint::default();
            let cm = ColorMatrix::new(
                transform.color_transform.r_mult.to_f32(), 0., 0., 0., transform.color_transform.r_add.into(),
                0., transform.color_transform.g_mult.to_f32(), 0., 0., transform.color_transform.g_add.into(),
                0., 0., transform.color_transform.b_mult.to_f32(), 0., transform.color_transform.b_add.into(),
                0., 0., 0., transform.color_transform.a_mult.to_f32(), transform.color_transform.a_add.into()
            );
            paint.set_color_filter(Some(skia_safe::color_filters::matrix(&cm)));

            can.draw_image(bm, (0, 0), Some(&paint));

            can.reset_matrix();
        }
    }

    fn draw_rect(&mut self, color: Color, matrix: &Matrix) {
        let can = self.surface.canvas();

        let mut paint = Paint::default();
        let col = skia_safe::Color::from_argb(color.a, color.r, color.g, color.b);
        paint.set_color(col);
        paint.set_stroke_width(1.0);

        let mut mat = skia_safe::Matrix::new_identity();
        let (tx, ty) = (transform.matrix.tx.to_pixels() as f32, transform.matrix.ty.to_pixels() as f32);

        mat.set_translate_x(tx);
        mat.set_translate_y(ty);
        mat.set_skew_x(matrix.c);
        mat.set_skew_y(matrix.b);
        mat.set_scale_y(matrix.d);
        mat.set_scale_x(matrix.a);
        can.set_matrix(&mat);

        can.draw_rect(skia_safe::Rect::new(0., 0., 1., 1.), &paint);

        can.reset_matrix();
    }

    fn end_frame(&mut self) {

        let d = self.surface.image_snapshot();
        let pp = d.peek_pixels().unwrap();

        let mut pix2 = Vec::with_capacity((self.width * self.height) as usize);
        for y in 0..self.height {
        for x in 0..self.width {
                let c = pp.get_color(skia_safe::IPoint::new(x as i32, y as i32));
                let x = u32::from_be_bytes([0, c.r(), c.g(), c.b()]);
                pix2.push(x);
            }
        }

        self.blit.backend.update_framebuffer(&pix2, 0, 0, self.width as usize, 0, 0);
        self.blit.render();


        // let display = self.render.display;
        // let window = self.render.window;
        // let gc = self.render.gc;
        // let screen = self.render.screen;
        // let visual = self.render.visual;
        //
        // // let black = unsafe { x11::xlib::XBlackPixel(display, screen)};
        //
        // // unsafe { x11::xlib::XSetBackground(display, gc, black)};
        // // unsafe { x11::xlib::XSetForeground(display, gc, black)};
        // unsafe {  x11::xlib::XClearWindow(display, window)};
        //
        // // unsafe { x11::xlib::XDestroyWindow(display, window)};
        // //
        // // let w = 100u32;
        // // let h = 100u32;
        // // let mut data: Vec<u32> = vec![];
        // // data.resize((w * h) as usize, 0);
        // // let img = unsafe { x11::xlib::XCreateImage(display, visual, 32, x11::xlib::ZPixmap, 0, (&mut data[..]).as_mut_ptr() as *mut c_char, w, h, 32, (w as i32) * 4)};
        // // println!("Got image {:x}", img as usize);
        // //
        // // unsafe { x11::xlib::XPutImage(display, window, gc, img, 0, 0, 0, 0, h, w) };
        //
        // unsafe { x11::xlib::XFlush(display)};
    }

    fn push_mask(&mut self) {
        // todo!()
    }

    fn activate_mask(&mut self) {
        // todo!()
    }

    fn deactivate_mask(&mut self) {
        // todo!()
    }

    fn pop_mask(&mut self) {
        // todo!()
    }

    fn get_bitmap_pixels(&mut self, bitmap: BitmapHandle) -> Option<Bitmap> {
        todo!()
    }

    fn register_bitmap_raw(&mut self, width: u32, height: u32, rgba: Vec<u8>) -> Result<BitmapHandle, Error> {
        todo!()
    }

    fn update_texture(&mut self, bitmap: BitmapHandle, width: u32, height: u32, rgba: Vec<u8>) -> Result<BitmapHandle, Error> {
        todo!()
    }
}
