use crate::config::global::{
    CANVAS_BARCOEFF, CANVAS_BARWIDTH, CANVAS_MODEL, CANVAS_SPACING, CANVAS_STARTX, CANVAS_STARTY,
};
use crate::config::structs::InstanceInfo;
use crate::config::structs::Organes;
use crate::database::views::{ResultsPublic, ResultsPublicGroupes};
use crate::errors::{throw, ErrorKind, InstanceError};

use image::{DynamicImage, GenericImage, ImageFormat, Rgba};
use imageproc::drawing::draw_text_mut;
use rusttype::Scale;
use std::fs::File;
use std::io::BufReader;

// returns the message and its attachment (the image) as an array of bytes
pub fn gen_results_image(results: &ResultsPublic) -> Result<(String, Vec<u8>), InstanceError> {
    let g_instance = InstanceInfo::global();

    let mut img = image::load(
        BufReader::new(
            File::open(CANVAS_MODEL)
                .map_err(|e| {
                    eprintln!("Unable to open canvas model");
                    e
                })
                .map_err(|e| throw(ErrorKind::CritCanvasModelOpen, e.to_string()))?,
        ),
        ImageFormat::Png,
    )
    .map_err(|e| throw(ErrorKind::CritCanvasModelLoad, e.to_string()))?;

    // print general info to image
    print_layout(&mut img, g_instance, results);

    // 1. reorder groupes in result per best match
    let mut leaderboard = results.groupes.clone();
    leaderboard.sort_by(|a, b| {
        a.value_median
            .partial_cmp(&b.value_median)
            .expect("pubreport: There's no NaN in this set. Shouldn't happen.")
    });
    leaderboard.reverse();

    // 2. merge leading groups with their description from organes.json
    let mut leaderboard_desc: Vec<(Organes, ResultsPublicGroupes)> = Vec::new();

    for group in leaderboard {
        let leading_group_info = g_instance
            .acteurs_list
            .organes
            .iter()
            .find(|o| o.id == group.id);

        let leading_group_info = if let Some(v) = leading_group_info {
            v
        } else {
            return Err(throw(
                ErrorKind::CritNoLeadingGroup,
                format!("{:?}", leading_group_info),
            ));
        };

        leaderboard_desc.push((leading_group_info.clone(), group));
    }

    // filter groups having display==false
    for (i, group) in leaderboard_desc.iter().filter(|l| l.0.display).enumerate() {
        // convert color from string (hex) to rgba
        let group_color = if let Some(c) = color_hex2rgb(&group.0.color) {
            c
        } else {
            return Err(throw(
                ErrorKind::CritCanvasGroupColor,
                group.0.color.to_string(),
            ));
        };

        display_group(
            &mut img,
            i as u32,
            &group.0.abrev,
            group_color,
            group.1.value_median,
            g_instance,
        );
    }

    // leading group information for tweet text
    let leading_group_info = if let Some(l) = leaderboard_desc.iter().find(|l| l.0.display) {
        l
    } else {
        return Err(throw(
            ErrorKind::CritCanvasGroupInfo,
            format!("{:?}", leaderboard_desc),
        ));
    };

    //img.save("foo.png")?;
    //println!("Printed to file");

    let mut img_bytes: Vec<u8> = Vec::new();
    img.write_to(&mut img_bytes, image::ImageOutputFormat::Png)
        .map_err(|e| throw(ErrorKind::CritCanvasWriteBytes, e.to_string()))?;

    Ok((format!(
                "Statistiques de participation globales en date du {}\nComptabilisées : {} | Total : {}\nGroupe en tête : {} #{} ({:.1} %)\n#QuelParti https://quelparti.fr\n",
                results.global.generated_at.format("%d/%m/%Y"),
                results.global.participations.valid,
                results.global.participations.total,
                leading_group_info.0.name,
                leading_group_info.0.abrev,
                leading_group_info.1.value_median,
    ),
    img_bytes))
}

fn display_group(
    img: &mut DynamicImage,
    index: u32,
    abbr: &str,
    color: Rgba<u8>,
    pct: f32,
    g_instance: &InstanceInfo,
) {
    let text_color = Rgba([247, 247, 247, 255]);

    draw_text_mut(
        img,
        text_color,
        CANVAS_STARTX - 120,
        CANVAS_STARTY - 5 + index * CANVAS_SPACING,
        Scale { x: 21.0, y: 21.0 },
        &g_instance.results_font,
        abbr,
    );

    for i in 0..(pct * CANVAS_BARCOEFF) as u32 {
        for j in 0..CANVAS_BARWIDTH {
            img.put_pixel(
                CANVAS_STARTX + i,
                CANVAS_STARTY + j + index * CANVAS_SPACING,
                color,
            );
        }
    }

    for j in 0..(CANVAS_BARWIDTH - 2) {
        img.put_pixel(
            CANVAS_STARTX - 1,
            CANVAS_STARTY + 1 + j + index * CANVAS_SPACING,
            color,
        );
        img.put_pixel(
            CANVAS_STARTX + (pct * CANVAS_BARCOEFF) as u32,
            CANVAS_STARTY + 1 + j + index * CANVAS_SPACING,
            color,
        );
    }

    for j in 0..(CANVAS_BARWIDTH - 4) {
        img.put_pixel(
            CANVAS_STARTX - 2,
            CANVAS_STARTY + 2 + j + index * CANVAS_SPACING,
            color,
        );
        img.put_pixel(
            CANVAS_STARTX + 1 + (pct * CANVAS_BARCOEFF) as u32,
            CANVAS_STARTY + 2 + j + index * CANVAS_SPACING,
            color,
        );
    }

    draw_text_mut(
        img,
        text_color,
        CANVAS_STARTX + (pct * CANVAS_BARCOEFF) as u32 + 20,
        CANVAS_STARTY - 5 + index * CANVAS_SPACING,
        Scale { x: 21.0, y: 21.0 },
        &g_instance.results_font,
        &format!("{:.1} %", pct),
    );
}

fn print_layout(img: &mut DynamicImage, g_instance: &InstanceInfo, results: &ResultsPublic) {
    let text_color = Rgba([247, 247, 247, 255]);

    //let mut img = RgbaImage::new(960, 540);

    // Drawing title
    draw_text_mut(
        img,
        text_color,
        300,
        60,
        Scale { x: 54.0, y: 54.0 },
        &g_instance.results_font,
        "Résultat global",
    );

    // Drawing subtitle (generated_at)
    draw_text_mut(
        img,
        text_color,
        300,
        120,
        Scale { x: 28.0, y: 28.0 },
        &g_instance.results_font,
        &format!(
            "En date du {}",
            results.global.generated_at.format("%d/%m/%Y")
        ),
    );

    // Drawing subtitle (started_at)
    draw_text_mut(
        img,
        text_color,
        300,
        148,
        Scale { x: 20.0, y: 20.0 },
        &g_instance.results_font,
        &format!("Depuis le {}", results.global.started_at.format("%d/%m/%Y")),
    );

    // Drawing total_participations
    draw_text_mut(
        img,
        text_color,
        680,
        70,
        Scale { x: 18.0, y: 18.0 },
        &g_instance.results_font,
        &format!("{} participations", results.global.participations.total),
    );

    // Drawing valid_participations
    draw_text_mut(
        img,
        text_color,
        680,
        90,
        Scale { x: 18.0, y: 18.0 },
        &g_instance.results_font,
        &format!(
            "dont {} comptabilisées",
            results.global.participations.valid
        ),
    );

    // Drawing scrutin mode label
    draw_text_mut(
        img,
        text_color,
        680,
        120,
        Scale { x: 18.0, y: 18.0 },
        &g_instance.results_font,
        "Mode de scrutin :",
    );

    // Drawing selected scrutin mode
    draw_text_mut(
        img,
        text_color,
        680,
        142,
        Scale { x: 21.0, y: 21.0 },
        &g_instance.results_font,
        "Meilleure médiane",
    );
}

fn color_hex2rgb(color: &str) -> Option<Rgba<u8>> {
    let r = u8::from_str_radix(&color[1..=2], 16).ok()?;
    let g = u8::from_str_radix(&color[3..=4], 16).ok()?;
    let b = u8::from_str_radix(&color[5..=6], 16).ok()?;

    Some(Rgba([r, g, b, 255]))
}
