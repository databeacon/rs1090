#![allow(rustdoc::missing_crate_level_docs)]

use std::collections::HashMap;

use pyo3::exceptions::{PyAssertionError, PyValueError};
use pyo3::prelude::*;
use rayon::prelude::*;
use regex::Regex;
use rs1090::data::patterns::PATTERNS;
use rs1090::data::tail::tail;
use rs1090::decode::bds::bds05::AirbornePosition;
use rs1090::decode::bds::bds10::DataLinkCapability;
use rs1090::decode::bds::bds17::CommonUsageGICBCapabilityReport;
use rs1090::decode::bds::bds18::GICBCapabilityReportPart1;
use rs1090::decode::bds::bds19::GICBCapabilityReportPart2;
use rs1090::decode::bds::bds20::AircraftIdentification;
use rs1090::decode::bds::bds21::AircraftAndAirlineRegistrationMarkings;
use rs1090::decode::bds::bds30::ACASResolutionAdvisory;
use rs1090::decode::bds::bds40::SelectedVerticalIntention;
use rs1090::decode::bds::bds44::MeteorologicalRoutineAirReport;
use rs1090::decode::bds::bds45::MeteorologicalHazardReport;
use rs1090::decode::bds::bds50::TrackAndTurnReport;
use rs1090::decode::bds::bds60::HeadingAndSpeedReport;
use rs1090::decode::bds::bds65::AircraftOperationStatus;
use rs1090::decode::cpr::{decode_positions, Position};
use rs1090::decode::flarm::Flarm;
use rs1090::prelude::*;

#[pyfunction]
fn decode_1090(msg: String) -> PyResult<Vec<u8>> {
    let bytes = hex::decode(msg)
        .map_err(|e| DecodeError(DekuError::Parse(e.to_string().into())))?;
    if let Ok((_, msg)) = Message::from_bytes((&bytes, 0)) {
        let pkl = serde_pickle::to_vec(&msg, Default::default())
            .map_err(|e| DecodeError(DekuError::Parse(e.to_string().into())))?;
        Ok(pkl)
    } else {
        Ok([128, 4, 78, 46].to_vec()) // None
    }
}
struct DecodeError(DekuError);

impl From<DecodeError> for PyErr {
    fn from(error: DecodeError) -> Self {
        match error.0 {
            DekuError::Assertion(msg) => PyAssertionError::new_err(msg),
            _ => PyValueError::new_err(error.0.to_string()),
        }
    }
}

#[pyfunction]
fn decode_bds05(msg: String) -> PyResult<Vec<u8>> {
    let bytes = hex::decode(msg)
        .map_err(|e| DecodeError(DekuError::Parse(e.to_string().into())))?;
    let tc = &bytes[4] >> 3;
    if (9..22).contains(&tc) && tc != 19 {
        match AirbornePosition::from_bytes((&bytes[4..], 0)) {
            Ok((_, msg)) => {
                let pkl = serde_pickle::to_vec(&msg, Default::default())
                    .map_err(|e| {
                        DecodeError(DekuError::Parse(e.to_string().into()))
                    })?;
                Ok(pkl)
            }
            Err(e) => Err(DecodeError(e).into()),
        }
    } else {
        let msg = format!(
            "Invalid typecode {} for BDS 0,5 (9 to 18 or 20 to 22)",
            tc
        );
        Err(PyAssertionError::new_err(msg))
    }
}

#[pyfunction]
fn decode_bds10(msg: String) -> PyResult<Vec<u8>> {
    let bytes = hex::decode(msg)
        .map_err(|e| DecodeError(DekuError::Parse(e.to_string().into())))?;
    match DataLinkCapability::from_bytes((&bytes[4..], 0)) {
        Ok((_, msg)) => {
            let pkl = serde_pickle::to_vec(&msg, Default::default()).map_err(
                |e| DecodeError(DekuError::Parse(e.to_string().into())),
            )?;
            Ok(pkl)
        }
        Err(e) => Err(DecodeError(e).into()),
    }
}

#[pyfunction]
fn decode_bds17(msg: String) -> PyResult<Vec<u8>> {
    let bytes = hex::decode(msg)
        .map_err(|e| DecodeError(DekuError::Parse(e.to_string().into())))?;
    match CommonUsageGICBCapabilityReport::from_bytes((&bytes[4..], 0)) {
        Ok((_, msg)) => {
            let pkl = serde_pickle::to_vec(&msg, Default::default()).map_err(
                |e| DecodeError(DekuError::Parse(e.to_string().into())),
            )?;
            Ok(pkl)
        }
        Err(e) => Err(DecodeError(e).into()),
    }
}

#[pyfunction]
fn decode_bds18(msg: String) -> PyResult<Vec<u8>> {
    let bytes = hex::decode(msg)
        .map_err(|e| DecodeError(DekuError::Parse(e.to_string().into())))?;
    match GICBCapabilityReportPart1::from_bytes((&bytes[4..], 0)) {
        Ok((_, msg)) => {
            let pkl = serde_pickle::to_vec(&msg, Default::default()).map_err(
                |e| DecodeError(DekuError::Parse(e.to_string().into())),
            )?;
            Ok(pkl)
        }
        Err(e) => Err(DecodeError(e).into()),
    }
}

#[pyfunction]
fn decode_bds19(msg: String) -> PyResult<Vec<u8>> {
    let bytes = hex::decode(msg)
        .map_err(|e| DecodeError(DekuError::Parse(e.to_string().into())))?;
    match GICBCapabilityReportPart2::from_bytes((&bytes[4..], 0)) {
        Ok((_, msg)) => {
            let pkl = serde_pickle::to_vec(&msg, Default::default()).map_err(
                |e| DecodeError(DekuError::Parse(e.to_string().into())),
            )?;
            Ok(pkl)
        }
        Err(e) => Err(DecodeError(e).into()),
    }
}

#[pyfunction]
fn decode_bds20(msg: String) -> PyResult<Vec<u8>> {
    let bytes = hex::decode(msg)
        .map_err(|e| DecodeError(DekuError::Parse(e.to_string().into())))?;
    match AircraftIdentification::from_bytes((&bytes[4..], 0)) {
        Ok((_, msg)) => {
            let pkl = serde_pickle::to_vec(&msg, Default::default()).map_err(
                |e| DecodeError(DekuError::Parse(e.to_string().into())),
            )?;
            Ok(pkl)
        }
        Err(e) => Err(DecodeError(e).into()),
    }
}

#[pyfunction]
fn decode_bds21(msg: String) -> PyResult<Vec<u8>> {
    let bytes = hex::decode(msg)
        .map_err(|e| DecodeError(DekuError::Parse(e.to_string().into())))?;
    match AircraftAndAirlineRegistrationMarkings::from_bytes((&bytes[4..], 0)) {
        Ok((_, msg)) => {
            let pkl = serde_pickle::to_vec(&msg, Default::default()).map_err(
                |e| DecodeError(DekuError::Parse(e.to_string().into())),
            )?;
            Ok(pkl)
        }
        Err(e) => Err(DecodeError(e).into()),
    }
}

#[pyfunction]
fn decode_bds30(msg: String) -> PyResult<Vec<u8>> {
    let bytes = hex::decode(msg)
        .map_err(|e| DecodeError(DekuError::Parse(e.to_string().into())))?;
    match ACASResolutionAdvisory::from_bytes((&bytes[4..], 0)) {
        Ok((_, msg)) => {
            let pkl = serde_pickle::to_vec(&msg, Default::default()).map_err(
                |e| DecodeError(DekuError::Parse(e.to_string().into())),
            )?;
            Ok(pkl)
        }
        Err(e) => Err(DecodeError(e).into()),
    }
}

#[pyfunction]
fn decode_bds40(msg: String) -> PyResult<Vec<u8>> {
    let bytes = hex::decode(msg)
        .map_err(|e| DecodeError(DekuError::Parse(e.to_string().into())))?;
    match SelectedVerticalIntention::from_bytes((&bytes[4..], 0)) {
        Ok((_, msg)) => {
            let pkl = serde_pickle::to_vec(&msg, Default::default()).map_err(
                |e| DecodeError(DekuError::Parse(e.to_string().into())),
            )?;
            Ok(pkl)
        }
        Err(e) => Err(DecodeError(e).into()),
    }
}

#[pyfunction]
fn decode_bds44(msg: String) -> PyResult<Vec<u8>> {
    let bytes = hex::decode(msg)
        .map_err(|e| DecodeError(DekuError::Parse(e.to_string().into())))?;
    match MeteorologicalRoutineAirReport::from_bytes((&bytes[4..], 0)) {
        Ok((_, msg)) => {
            let pkl = serde_pickle::to_vec(&msg, Default::default()).map_err(
                |e| DecodeError(DekuError::Parse(e.to_string().into())),
            )?;
            Ok(pkl)
        }
        Err(e) => Err(DecodeError(e).into()),
    }
}
#[pyfunction]
fn decode_bds45(msg: String) -> PyResult<Vec<u8>> {
    let bytes = hex::decode(msg)
        .map_err(|e| DecodeError(DekuError::Parse(e.to_string().into())))?;
    match MeteorologicalHazardReport::from_bytes((&bytes[4..], 0)) {
        Ok((_, msg)) => {
            let pkl = serde_pickle::to_vec(&msg, Default::default()).map_err(
                |e| DecodeError(DekuError::Parse(e.to_string().into())),
            )?;
            Ok(pkl)
        }
        Err(e) => Err(DecodeError(e).into()),
    }
}

#[pyfunction]
fn decode_bds50(msg: String) -> PyResult<Vec<u8>> {
    let bytes = hex::decode(msg)
        .map_err(|e| DecodeError(DekuError::Parse(e.to_string().into())))?;
    match TrackAndTurnReport::from_bytes((&bytes[4..], 0)) {
        Ok((_, msg)) => {
            let pkl = serde_pickle::to_vec(&msg, Default::default()).map_err(
                |e| DecodeError(DekuError::Parse(e.to_string().into())),
            )?;
            Ok(pkl)
        }
        Err(e) => Err(DecodeError(e).into()),
    }
}

#[pyfunction]
fn decode_bds60(msg: String) -> PyResult<Vec<u8>> {
    let bytes = hex::decode(msg)
        .map_err(|e| DecodeError(DekuError::Parse(e.to_string().into())))?;
    match HeadingAndSpeedReport::from_bytes((&bytes[4..], 0)) {
        Ok((_, msg)) => {
            let pkl = serde_pickle::to_vec(&msg, Default::default()).map_err(
                |e| DecodeError(DekuError::Parse(e.to_string().into())),
            )?;
            Ok(pkl)
        }
        Err(e) => Err(DecodeError(e).into()),
    }
}

#[pyfunction]
fn decode_bds65(msg: String) -> PyResult<Vec<u8>> {
    let bytes = hex::decode(msg)
        .map_err(|e| DecodeError(DekuError::Parse(e.to_string().into())))?;
    let tc = &bytes[4] >> 3;
    let enum_id = &bytes[4] & 0b111;
    match (tc, enum_id) {
        (31, id) if id < 2 => {
            match AircraftOperationStatus::from_bytes((&bytes[4..], 0)) {
                Ok((_, msg)) => {
                    let pkl = serde_pickle::to_vec(&msg, Default::default())
                        .map_err(|e| {
                            DecodeError(DekuError::Parse(e.to_string().into()))
                        })?;
                    Ok(pkl)
                }
                Err(e) => Err(DecodeError(e).into()),
            }
        }
        _ => Err(PyAssertionError::new_err(format!(
            "Invalid typecode {} (31) or category {} (0 or 1) for BDS 6,5",
            tc, enum_id
        ))),
    }
}

#[pyfunction]
fn decode_1090_vec(msgs_set: Vec<Vec<String>>) -> PyResult<Vec<u8>> {
    let res: Vec<Option<Message>> = msgs_set
        .par_iter()
        .map(|msgs| {
            msgs.iter()
                .map(|msg| {
                    let bytes = hex::decode(msg)
                        .map_err(|e| {
                            DecodeError(DekuError::Parse(e.to_string().into()))
                        })
                        .ok()?;
                    if let Ok((_, msg)) = Message::from_bytes((&bytes, 0)) {
                        Some(msg)
                    } else {
                        None
                    }
                })
                .collect()
        })
        .flat_map(|v: Vec<Option<Message>>| v)
        .collect();
    let pkl = serde_pickle::to_vec(&res, Default::default())
        .map_err(|e| DecodeError(DekuError::Parse(e.to_string().into())))?;
    Ok(pkl)
}

#[pyfunction]
#[pyo3(signature = (msgs_set, ts_set, reference=None))]
fn decode_1090t_vec(
    msgs_set: Vec<Vec<String>>,
    ts_set: Vec<Vec<f64>>,
    reference: Option<[f64; 2]>,
) -> PyResult<Vec<u8>> {
    let mut res: Vec<TimedMessage> = msgs_set
        .par_iter()
        .zip(ts_set)
        .map(|(msgs, ts)| {
            msgs.iter()
                .zip(ts)
                .filter_map(|(msg, timestamp)| {
                    let bytes = hex::decode(msg)
                        .map_err(|e| {
                            DecodeError(DekuError::Parse(e.to_string().into()))
                        })
                        .ok()?;
                    if let Ok((_, message)) = Message::from_bytes((&bytes, 0)) {
                        Some(TimedMessage {
                            timestamp,
                            frame: bytes,
                            message: Some(message),
                            metadata: vec![],
                            decode_time: None,
                        })
                    } else {
                        None
                    }
                })
                .collect()
        })
        .flat_map(|v: Vec<TimedMessage>| v)
        .collect();

    let position = reference.map(|[latitude, longitude]| Position {
        latitude,
        longitude,
    });
    decode_positions(&mut res, position, &None);

    let pkl = serde_pickle::to_vec(&res, Default::default())
        .map_err(|e| DecodeError(DekuError::Parse(e.to_string().into())))?;
    Ok(pkl)
}

#[pyfunction]
fn decode_flarm(
    msg: String,
    ts: u32,
    reflat: f64,
    reflon: f64,
) -> PyResult<Vec<u8>> {
    let bytes = hex::decode(msg)
        .map_err(|e| DecodeError(DekuError::Parse(e.to_string().into())))?;
    let reference = [reflat, reflon];
    if let Ok(msg) = Flarm::from_record(ts, &reference, &bytes) {
        let pkl = serde_pickle::to_vec(&msg, Default::default())
            .map_err(|e| DecodeError(DekuError::Parse(e.to_string().into())))?;
        Ok(pkl)
    } else {
        Ok([128, 4, 78, 46].to_vec()) // None
    }
}

#[pyfunction]
fn decode_flarm_vec(
    msgs_set: Vec<Vec<String>>,
    ts_set: Vec<Vec<u32>>,
    ref_lat: Vec<Vec<f64>>,
    ref_lon: Vec<Vec<f64>>,
) -> PyResult<Vec<u8>> {
    let reference: Vec<Vec<[f64; 2]>> = ref_lat
        .iter()
        .zip(ref_lon.iter())
        .map(|(lat, lon)| {
            lat.iter()
                .zip(lon.iter())
                .map(|(lat, lon)| [*lat, *lon])
                .collect()
        })
        .collect();
    let res: Vec<Flarm> = msgs_set
        .par_iter()
        .zip(ts_set)
        .zip(reference)
        .map(|((msgs, ts), reference)| {
            msgs.iter()
                .zip(ts)
                .zip(reference)
                .filter_map(|((msg, timestamp), reference)| {
                    let bytes = hex::decode(msg)
                        .map_err(|e| {
                            DecodeError(DekuError::Parse(e.to_string().into()))
                        })
                        .ok()?;
                    if let Ok(flarm) =
                        Flarm::from_record(timestamp, &reference, &bytes)
                    {
                        Some(flarm)
                    } else {
                        None
                    }
                })
                .collect()
        })
        .flat_map(|v: Vec<Flarm>| v)
        .collect();

    let pkl = serde_pickle::to_vec(&res, Default::default())
        .map_err(|e| DecodeError(DekuError::Parse(e.to_string().into())))?;
    Ok(pkl)
}

#[pyfunction]
#[pyo3(signature = (icao24, registration=None))]
fn aircraft_information(
    icao24: &str,
    registration: Option<&str>,
) -> PyResult<HashMap<String, String>> {
    let mut reg = HashMap::<String, String>::new();
    let hexid = u32::from_str_radix(icao24, 16)?;

    reg.insert("icao24".to_string(), icao24.to_lowercase());

    if let Some(tail) = tail(hexid) {
        reg.insert("registration".to_string(), tail);
    }
    if let Some(tail) = registration {
        reg.insert("registration".to_string(), tail.to_string());
    }

    if let Some(pattern) = &PATTERNS.registers.iter().find(|elt| {
        if let Some(start) = &elt.start {
            if let Some(end) = &elt.end {
                let start = u32::from_str_radix(&start[2..], 16).unwrap();
                let end = u32::from_str_radix(&end[2..], 16).unwrap();
                return (hexid >= start) & (hexid <= end);
            }
        }
        false
    }) {
        reg.insert("country".to_string(), pattern.country.to_string());
        reg.insert("flag".to_string(), pattern.flag.to_string());
        if let Some(p) = &pattern.pattern {
            reg.insert("pattern".to_string(), p.to_string());
        }
        if let Some(comment) = &pattern.comment {
            reg.insert("comment".to_string(), comment.to_string());
        }

        if let Some(tail) = reg.get("registration") {
            if let Some(categories) = &pattern.categories {
                if let Some(cat) = categories.iter().find(|elt| {
                    let re = Regex::new(&elt.pattern).unwrap();
                    re.is_match(tail)
                }) {
                    reg.insert("pattern".to_string(), cat.pattern.to_string());
                    if let Some(category) = &cat.category {
                        reg.insert(
                            "category".to_string(),
                            category.to_string(),
                        );
                    }
                    if let Some(country) = &cat.country {
                        reg.insert("country".to_string(), country.to_string());
                    }
                    if let Some(flag) = &cat.flag {
                        reg.insert("flag".to_string(), flag.to_string());
                    }
                }
            }
        }
    } else {
        reg.insert("country".to_string(), "Unknown".to_string());
        reg.insert("flag".to_string(), "🏳".to_string());
    }

    Ok(reg)
}

/// A Python module implemented in Rust.
#[pymodule]
fn _rust(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Decoding functions
    m.add_function(wrap_pyfunction!(decode_1090, m)?)?;
    m.add_function(wrap_pyfunction!(decode_1090_vec, m)?)?;
    m.add_function(wrap_pyfunction!(decode_1090t_vec, m)?)?;
    m.add_function(wrap_pyfunction!(decode_flarm, m)?)?;
    m.add_function(wrap_pyfunction!(decode_flarm_vec, m)?)?;

    // Comm-B BDS inference
    m.add_function(wrap_pyfunction!(decode_bds05, m)?)?;
    m.add_function(wrap_pyfunction!(decode_bds10, m)?)?;
    m.add_function(wrap_pyfunction!(decode_bds17, m)?)?;
    m.add_function(wrap_pyfunction!(decode_bds18, m)?)?;
    m.add_function(wrap_pyfunction!(decode_bds19, m)?)?;
    m.add_function(wrap_pyfunction!(decode_bds20, m)?)?;
    m.add_function(wrap_pyfunction!(decode_bds21, m)?)?;
    m.add_function(wrap_pyfunction!(decode_bds30, m)?)?;
    m.add_function(wrap_pyfunction!(decode_bds40, m)?)?;
    m.add_function(wrap_pyfunction!(decode_bds44, m)?)?;
    m.add_function(wrap_pyfunction!(decode_bds45, m)?)?;
    m.add_function(wrap_pyfunction!(decode_bds50, m)?)?;
    m.add_function(wrap_pyfunction!(decode_bds60, m)?)?;
    m.add_function(wrap_pyfunction!(decode_bds65, m)?)?;

    // icao24 functions
    m.add_function(wrap_pyfunction!(aircraft_information, m)?)?;

    Ok(())
}
