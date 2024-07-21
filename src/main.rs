extern crate opencv;

use opencv::{
    core::{self, Point, Point2f, Scalar, Size, Mat, Rect},
    imgproc::{self, ContourApproximationModes, RetrievalModes},
    highgui,
    imgcodecs,
    prelude::*,
    types::{VectorOfVectorOfPoint, VectorOfPoint2f},
};

fn main() -> opencv::Result<()> {
    // Define input height and width for the cropped area
    let input_height = 300; // Adjust this value as needed
    let input_width = 300; // Adjust this value as needed

    // Step 1: Load the image
    let img = imgcodecs::imread("test1.png", imgcodecs::IMREAD_COLOR)?;
    if img.empty() {
        return Err(opencv::Error::new(opencv::core::StsError, "Error loading image"));
    }

    // Step 2: Convert to grayscale
    let mut gray = Mat::default();
    imgproc::cvt_color(&img, &mut gray, imgproc::COLOR_BGR2GRAY, 0)?;

    // Display the grayscale image
    highgui::imshow("Grayscale Image", &gray)?;
    highgui::wait_key(0)?;

    // Step 3: Apply binary threshold
    let mut binary = Mat::default();
    imgproc::threshold(&gray, &mut binary, 0.0, 255.0, imgproc::THRESH_BINARY_INV + imgproc::THRESH_OTSU)?;

    // Display the binary image
    highgui::imshow("Binary Image", &binary)?;
    highgui::wait_key(0)?;

    // Step 4: Find contours
    let mut contours = VectorOfVectorOfPoint::new();
    imgproc::find_contours(
        &binary,
        &mut contours,
        RetrievalModes::RETR_EXTERNAL.into(),
        ContourApproximationModes::CHAIN_APPROX_SIMPLE.into(),
        Point::new(0, 0),
    )?;

    println!("Number of contours found: {}", contours.len());

    // Step 5: Iterate through contours and find the rotated rectangle with the largest area
    let mut largest_area = 0.0;
    let mut best_rect = None;
    let mut best_contour = None;

    for i in 0..contours.len() {
        let contour = contours.get(i)?;
        if contour.len() < 3 {
            continue;
        }
        let rect = imgproc::min_area_rect(&contour)?;
        let area = rect.size.width * rect.size.height;
        println!("Contour {}: Area = {}", i, area);
        if area > largest_area {
            largest_area = area;
            best_rect = Some(rect);
            best_contour = Some(contour);
        }
    }

    // Step 6: Process the best rectangle if found
    if let Some(rect) = best_rect {
        // Draw the best contour
        let mut contour_img = img.clone();
        if let Some(contour) = best_contour {
            imgproc::draw_contours(
                &mut contour_img,
                &VectorOfVectorOfPoint::from_iter(vec![contour]),
                -1,
                Scalar::new(0.0, 255.0, 0.0, 0.0),
                2,
                imgproc::LINE_8,
                &core::no_array(),
                i32::MAX,
                Point::new(0, 0)
            )?;
            highgui::imshow("Best Contour", &contour_img)?;
            highgui::wait_key(0)?;
        }

        // Calculate the rotation angle and center
        let angle = rect.angle;
        let center = Point2f::new(rect.center.x as f32, rect.center.y as f32);

        println!("Best rectangle: Center = ({}, {}), Angle = {}", center.x, center.y, angle);

        // Rotate the image to align the rectangle
        let rotation_matrix = imgproc::get_rotation_matrix_2d(center, angle.into(), 1.0)?;
        println!("Rotation Matrix: {:?}", rotation_matrix);

        // Determine the bounding box size for the rotated image
        let mut bbox = Mat::default();
        imgproc::warp_affine(
            &img,
            &mut bbox,
            &rotation_matrix,
            Size::new(img.cols(), img.rows()),
            imgproc::INTER_LINEAR,
            core::BORDER_CONSTANT,
            Scalar::all(255.0),
        )?;

        let bbox_size = core::Rect::new(0, 0, bbox.cols(), bbox.rows());
        println!("Bounding Box Size: {:?}", bbox_size);

        let mut rotated = Mat::new_size_with_default(bbox_size.size(), img.typ(), Scalar::all(255.0))?;
        imgproc::warp_affine(
            &img,
            &mut rotated,
            &rotation_matrix,
            bbox_size.size(),
            imgproc::INTER_LINEAR,
            core::BORDER_CONSTANT,
            Scalar::all(255.0),
        )?;

        // Display the rotated image
        highgui::imshow("Rotated Image", &rotated)?;
        highgui::wait_key(0)?;

        // Calculate the cropping rectangle
        let crop_x = (center.x as i32) - (input_width / 2);
        let crop_y = (center.y as i32) - (input_height / 2);

        let crop_rect = Rect::new(
            crop_x.max(0).min(rotated.cols() - input_width),
            crop_y.max(0).min(rotated.rows() - input_height),
            input_width.min(rotated.cols()),
            input_height.min(rotated.rows())
        );

        let cropped = Mat::roi(&rotated, crop_rect)?;

        // Display the cropped image
        highgui::imshow("Cropped Image", &cropped)?;
        highgui::wait_key(0)?;
    } else {
        println!("No suitable rectangle found.");
    }

    Ok(())
}
