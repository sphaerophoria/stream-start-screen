use std::{
    collections::HashMap,
    io::{BufRead, Error as IoError},
};

use thiserror::Error;

#[derive(Debug, Error)]
#[error("obj parse error")]
pub enum ObjParseError {
    FileRead(IoError),
    MissingType,
    MissingVertex,
    NonFloatVertex(std::num::ParseFloatError),
    MissingFaceVert,
    InvalidFaceVert(std::num::ParseIntError),
    InvalidFaceUv(std::num::ParseIntError),
    InvalidFaceNorm(std::num::ParseIntError),
    MissingTexCoord,
    NonFloatTexCoord(std::num::ParseFloatError),
}

#[repr(C)]
#[derive(Debug, Default)]
pub struct VertData {
    pub vert: [f32; 4],
    pub uv: [f32; 2],
    pub norm: [f32; 3],
}

impl VertData {
    const fn new() -> VertData {
        VertData {
            vert: [0.0; 4],
            uv: [0.0; 2],
            norm: [0.0; 3],
        }
    }
    pub const fn vert_offset() -> i32 {
        let obj = VertData::new();
        unsafe {
            (std::ptr::addr_of!(obj.vert) as *const u8)
                .offset_from(&obj as *const VertData as *const u8) as i32
        }
    }

    pub const fn uv_offset() -> i32 {
        let obj = VertData::new();
        unsafe {
            (std::ptr::addr_of!(obj.uv) as *const u8)
                .offset_from(&obj as *const VertData as *const u8) as i32
        }
    }

    pub const fn normal_offset() -> i32 {
        let obj = VertData::new();
        unsafe {
            (std::ptr::addr_of!(obj.norm) as *const u8)
                .offset_from(&obj as *const VertData as *const u8) as i32
        }
    }
}

#[derive(Debug)]
pub struct Mesh {
    pub vertices: Vec<VertData>,
    pub faces: Vec<[u32; 3]>,
}

impl Mesh {
    pub fn from_obj_file<R: BufRead>(r: R) -> Result<Mesh, ObjParseError> {
        let mut vertices = Vec::new();
        let mut faces = Vec::new();
        let mut tex_coords = Vec::new();
        let mut normals = Vec::new();

        for line in r.lines() {
            let line = line.map_err(ObjParseError::FileRead)?;
            let mut line_it = line.split_whitespace();

            let typ = line_it.next().ok_or(ObjParseError::MissingType)?;
            match typ {
                "v" => {
                    let v = parse_vertex(line_it)?;
                    vertices.push(v);
                }
                "f" => {
                    let v = parse_face(line_it)?;
                    faces.push(v);
                }
                "vt" => {
                    let v = parse_tex_coord(line_it)?;
                    tex_coords.push(v);
                }
                "vn" => {
                    let v = parse_vertex_3(line_it)?;
                    normals.push(v);
                }
                t => {
                    println!("Unsupported type {t}");
                }
            }
        }

        Ok(obj_data_to_mesh(&vertices, &tex_coords, &normals, &faces))
    }
}

fn parse_vertex_n<'a, It: Iterator<Item = &'a str>>(
    it: &mut It,
    data: &mut [f32],
) -> Result<(), ObjParseError> {
    for i in 0..data.len() {
        data[i] = it
            .next()
            .ok_or(ObjParseError::MissingVertex)?
            .parse()
            .map_err(ObjParseError::NonFloatVertex)?;
    }
    Ok(())
}

fn parse_vertex_3<'a, It: Iterator<Item = &'a str>>(mut it: It) -> Result<[f32; 3], ObjParseError> {
    let mut res = [0f32; 3];
    parse_vertex_n(&mut it, &mut res)?;
    Ok(res)
}

fn parse_vertex<'a, It: Iterator<Item = &'a str>>(mut it: It) -> Result<[f32; 4], ObjParseError> {
    let mut res = [0f32; 4];

    parse_vertex_n(&mut it, &mut res[0..3])?;

    res[3] = match it.next() {
        Some(v) => v.parse().map_err(ObjParseError::NonFloatVertex)?,
        None => 1.0,
    };

    Ok(res)
}

fn parse_tex_coord<'a, It: Iterator<Item = &'a str>>(
    mut it: It,
) -> Result<[f32; 2], ObjParseError> {
    let mut res = [0f32; 2];

    for i in 0..2 {
        res[i] = it
            .next()
            .ok_or(ObjParseError::MissingTexCoord)?
            .parse()
            .map_err(ObjParseError::NonFloatTexCoord)?;
    }

    Ok(res)
}

fn parse_face<'a, It: Iterator<Item = &'a str>>(
    mut it: It,
) -> Result<[FaceIndices; 3], ObjParseError> {
    let mut ret = [
        FaceIndices {
            vert: 0,
            uv: 0,
            norm: 0,
        },
        FaceIndices {
            vert: 0,
            uv: 0,
            norm: 0,
        },
        FaceIndices {
            vert: 0,
            uv: 0,
            norm: 0,
        },
    ];

    for i in 0..3 {
        let face = it.next().ok_or(ObjParseError::MissingFaceVert)?;
        let mut face_it = face.split('/');
        let vert_id = face_it
            .next()
            .expect("first element doesn't exist for obj face");
        ret[i].vert = vert_id
            .parse::<u32>()
            .map_err(ObjParseError::InvalidFaceVert)?
            - 1u32;

        let tex_id = face_it
            .next()
            .expect("second element doesn't exist for obj face");
        ret[i].uv = tex_id
            .parse::<u32>()
            .map_err(ObjParseError::InvalidFaceUv)?
            - 1u32;

        let norm_id = face_it
            .next()
            .expect("third element doesn't exist for obj face");
        ret[i].norm = norm_id
            .parse::<u32>()
            .map_err(ObjParseError::InvalidFaceNorm)?
            - 1u32;
    }

    Ok(ret)
}

#[derive(Debug, Hash, Clone, Copy, Eq, PartialEq)]
struct FaceIndices {
    vert: u32,
    uv: u32,
    norm: u32,
}

fn obj_data_to_mesh(
    in_vertices: &[[f32; 4]],
    in_uvs: &[[f32; 2]],
    in_normals: &[[f32; 3]],
    in_faces: &[[FaceIndices; 3]],
) -> Mesh {
    type MergedIndex = u32;

    let mut mapping: HashMap<FaceIndices, MergedIndex> = HashMap::new();
    // If we've seen this, take the index of vert_and_uv for that pair
    // If we haven't seen it, create a new vert/uv pair and push into vert_and_uv
    let mut output_vert_and_uv = Vec::new();
    let mut output_faces = Vec::new();

    for face in in_faces {
        let mut output_face = [0u32; 3];

        for (i, vert) in face.iter().enumerate() {
            let entry = mapping.entry(*vert).or_insert_with(|| {
                output_vert_and_uv.push(VertData {
                    vert: in_vertices[vert.vert as usize],
                    uv: in_uvs[vert.uv as usize],
                    norm: in_normals[vert.norm as usize],
                });

                (output_vert_and_uv.len() - 1).try_into().unwrap()
            });

            output_face[i] = *entry;
        }

        output_faces.push(output_face);
    }

    Mesh {
        vertices: output_vert_and_uv,
        faces: output_faces,
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_valid_vertex_parse() {
        match parse_vertex("1.0 2.0 3.0".split_whitespace()) {
            Ok(v) => {
                let expected = [1.0, 2.0, 3.0, 1.0];
                for (a, b) in v.into_iter().zip(expected) {
                    assert!((a - b).abs() < 0.0001);
                }
            }
            _ => panic!("Unexpected vertex parse failure"),
        };

        match parse_vertex("1.0 2.0 3.0 2.0".split_whitespace()) {
            Ok(v) => {
                let expected = [1.0, 2.0, 3.0, 2.0];
                for (a, b) in v.into_iter().zip(expected) {
                    assert!((a - b).abs() < 0.0001);
                }
            }
            _ => panic!("Unexpected vertex parse failure"),
        };
    }

    #[test]
    fn test_vertex_parse_invalid_float() {
        match parse_vertex("asdflka jdf".split_whitespace()) {
            Ok(_) => panic!("Invalid vertices should not parse"),
            Err(ObjParseError::NonFloatVertex(_)) => (),
            _ => panic!("Unexpected error type"),
        };

        match parse_vertex("1.0 jdf".split_whitespace()) {
            Ok(_) => panic!("Invalid vertices should not parse"),
            Err(ObjParseError::NonFloatVertex(_)) => (),
            _ => panic!("Unexpected error type"),
        };
    }

    #[test]
    fn test_face_parse_with_slashes() {
        match parse_face("1/2/3 2/3/4 3/4/5".split_whitespace()) {
            Ok(v) => assert_eq!(
                [
                    FaceIndices {
                        vert: 0,
                        uv: 1,
                        norm: 2
                    },
                    FaceIndices {
                        vert: 1,
                        uv: 2,
                        norm: 3
                    },
                    FaceIndices {
                        vert: 2,
                        uv: 3,
                        norm: 4
                    }
                ],
                v
            ),
            Err(e) => panic!("Unexpected face parse failure: {e:?}"),
        }
    }

    #[test]
    fn test_face_parse_not_enough_elems() {
        match parse_face("1/1/1 2/2/2".split_whitespace()) {
            Ok(_) => panic!("Face parse should have failed"),
            Err(ObjParseError::MissingFaceVert) => (),
            _ => panic!("Unexpected error for face parse"),
        }
    }

    #[test]
    fn test_face_parse_invalid_index() {
        match parse_face("1.1/1/1 2/2/2 3/3/3".split_whitespace()) {
            Ok(_) => panic!("Face parse should have failed"),
            Err(ObjParseError::InvalidFaceVert(_)) => (),
            e => panic!("Unexpected error for face parse: {e:?}"),
        }
        match parse_face("1/1.2/1 2/2/2 3/3/3".split_whitespace()) {
            Ok(_) => panic!("Face parse should have failed"),
            Err(ObjParseError::InvalidFaceUv(_)) => (),
            e => panic!("Unexpected error for face parse: {e:?}"),
        }

        match parse_face("1/1/1 2/2/2 asdf/3/3".split_whitespace()) {
            Ok(_) => panic!("Face parse should have failed"),
            Err(ObjParseError::InvalidFaceVert(_)) => (),
            e => panic!("Unexpected error for face parse: {e:?}"),
        }
    }
}
