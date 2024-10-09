use std::{collections::HashMap, error::Error, hash::Hash, process};

type EthnicityBits = u16;

#[derive(Debug, Clone, Copy)]
struct Ethnicities(EthnicityBits);

impl Ethnicities {
    const NATIVE_AMERICAN      : EthnicityBits = (0x0001);
    const SOUTHEAST_ASIAN      : EthnicityBits = (0x0002);
    const BLACK_AFRICAN_DESCENT: EthnicityBits = (0x0004);
    const EAST_ASIAN           : EthnicityBits = (0x0008);
    const HISPANIC_LATINO      : EthnicityBits = (0x0010);
    const MIDDLE_EASTERN       : EthnicityBits = (0x0020);
    const PACIFIC_ISLANDER     : EthnicityBits = (0x0040);
    const SOUTH_ASIAN          : EthnicityBits = (0x0080);
    const WHITE_CAUCASIAN      : EthnicityBits = (0x0100);
    const OTHER                : EthnicityBits = (0x8000);

    fn bits(&self) -> EthnicityBits {
        self.0
    }
}

impl From<EthnicityBits> for Ethnicities {
    fn from(bits: EthnicityBits) -> Self {
        Self(bits)
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum WhoLastReplied {
    You,
    Them,
    Met,
    None
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
enum Race {
    WhiteCaucasian,
    BlackAfrican,
    NativeAmerican,
    Asian,
    PacificIslander,
    Multiracial,
    Hispanic,
    Other
}

impl Race {
    fn aggregate(races: impl Iterator<Item = Self>) -> HashMap<Self, u32> {
        let mut race_counts = HashMap::from([
            (Race::WhiteCaucasian , 0),
            (Race::BlackAfrican   , 0),
            (Race::NativeAmerican , 0),
            (Race::Asian          , 0),
            (Race::PacificIslander, 0),
            (Race::Multiracial    , 0),
            (Race::Hispanic       , 0),
            (Race::Other          , 0)
        ]);
    
        for race in races {
            race_counts.entry(race).and_modify(|count| *count += 1).or_insert(1);
        }

        race_counts
    }
}

impl TryFrom<EthnicityBits> for Race {
    type Error = <Race as TryFrom<Ethnicities>>::Error;
    fn try_from(value: EthnicityBits) -> Result<Self, Self::Error> {
        Race::try_from(Ethnicities(value))
    }
}

impl TryFrom<Ethnicities> for Race {
    type Error = &'static str;
    fn try_from(value: Ethnicities) -> Result<Self, Self::Error> {
        const ASIAN_RACE_ETHNICITIES: EthnicityBits = Ethnicities::SOUTHEAST_ASIAN | Ethnicities::SOUTH_ASIAN | Ethnicities::EAST_ASIAN;
        const UNSUPPORTED_ETHNICITIES: EthnicityBits = Ethnicities::MIDDLE_EASTERN;

        let bits = value.bits();
        if bits == 0 {
            return Err("Ethnicity is empty");
        }

        let bits_ignore_not_supported = value.bits() & !UNSUPPORTED_ETHNICITIES;

        match bits_ignore_not_supported {
            0 => Err("Ethnicity only contains values that are not supported to be converted to a race distinction"),
            Ethnicities::NATIVE_AMERICAN => Ok(Self::NativeAmerican),
            Ethnicities::SOUTHEAST_ASIAN => Ok(Self::Asian),
            Ethnicities::BLACK_AFRICAN_DESCENT => Ok(Self::BlackAfrican),
            Ethnicities::EAST_ASIAN => Ok(Self::Asian),
            Ethnicities::PACIFIC_ISLANDER => Ok(Self::PacificIslander),
            Ethnicities::SOUTH_ASIAN => Ok(Self::Asian),
            Ethnicities::WHITE_CAUCASIAN => Ok(Self::WhiteCaucasian),
            Ethnicities::HISPANIC_LATINO => Ok(Self::Hispanic),
            Ethnicities::OTHER => Ok(Self::Other),
            _ => {
                // accept combinations of asian ethnicities
                if bits_ignore_not_supported & ASIAN_RACE_ETHNICITIES == bits_ignore_not_supported {
                    return Ok(Self::Asian);
                }

                if bits_ignore_not_supported & Ethnicities::HISPANIC_LATINO != 0 {
                    return Ok(Self::Hispanic);
                }

                // at this point bits must be part of at least two different disjoint sets of ethnicity groupings
                Ok(Self::Multiracial)
            }
        }
    }
}

#[derive(Debug, serde::Deserialize)]
struct HingeProfileCSVRecord {
    name: String,
    matched: u8,
    convo: u8,
    last_reply: String,
    specified: u8,
    native_american: u8,
    southeast_asian: u8,
    black_african_descent: u8,
    east_asian: u8,
    hispanic_latino: u8,
    middle_eastern: u8,
    pacific_islander: u8,
    south_asian: u8,
    white_caucasian: u8,
    other: u8
}

#[derive(Debug)]
struct HingeProfile {
    name: String,
    matched: bool,
    convo: bool,
    who_last_replied: WhoLastReplied,
    ethnicity_specified: bool,
    ethnicity: Ethnicities,
    race: Option<Race>,
}

impl TryFrom<HingeProfileCSVRecord> for HingeProfile {
    type Error = &'static str;
    fn try_from(value: HingeProfileCSVRecord) -> Result<Self, Self::Error> {
        let who_last_replied = match value.last_reply.as_str() {
            "You" => WhoLastReplied::You,
            "Them" => WhoLastReplied::Them,
            "None" => WhoLastReplied::None,
            "Met" => WhoLastReplied::Met,
            _ => return Err("Invalid value for Who Last Replied")
        };

        let ethnicity = Ethnicities(
            (if value.native_american       != 0  { Ethnicities::NATIVE_AMERICAN       } else { 0 }) |
            (if value.southeast_asian       != 0  { Ethnicities::SOUTHEAST_ASIAN       } else { 0 }) |
            (if value.black_african_descent != 0  { Ethnicities::BLACK_AFRICAN_DESCENT } else { 0 }) |
            (if value.east_asian            != 0  { Ethnicities::EAST_ASIAN            } else { 0 }) |
            (if value.hispanic_latino       != 0  { Ethnicities::HISPANIC_LATINO       } else { 0 }) |
            (if value.middle_eastern        != 0  { Ethnicities::MIDDLE_EASTERN        } else { 0 }) |
            (if value.pacific_islander      != 0  { Ethnicities::PACIFIC_ISLANDER      } else { 0 }) |
            (if value.south_asian           != 0  { Ethnicities::SOUTH_ASIAN           } else { 0 }) |
            (if value.white_caucasian       != 0  { Ethnicities::WHITE_CAUCASIAN       } else { 0 }) |
            (if value.other                 != 0  { Ethnicities::OTHER                 } else { 0 })
        );

        Ok(HingeProfile {
            name: value.name,
            matched: value.matched != 0,
            convo: value.convo != 0,
            who_last_replied,
            ethnicity_specified: value.specified != 0,
            ethnicity,
            race: ethnicity.try_into().ok()
        })
    }
}

#[derive(Debug, serde::Deserialize)]
struct CountyDemographicsCSVRecord {
    county: String,
    white_alone: u32,
    black_african_american_alone: u32,
    american_indian_alaska_native_alone: u32,
    asian_alone: u32,
    native_hawaiian_pacific_islander_alone: u32,
    some_other_race_alone: u32,
    two_or_more_races: u32,
    hispanic_latino: u32
}

#[derive(Debug, serde::Deserialize)]
struct CountyHispanicDemographicsCSVRecord {
    county: String,
    white_hispanic: u32,
    black_african_american_hispanic: u32,
    american_indian_alaska_native_hispanic: u32,
    asian_hispanic: u32,
    native_hawaiian_pacific_islander_hispanic: u32,
    some_other_race_hispanic: u32,
    two_or_more_races_hispanic: u32
}

fn example() -> Result<(), Box<dyn Error>> {

    // Source: https://datausa.io/profile/geo/cook-county-il#race_and_ethnicity
    // Source: https://datausa.io/profile/geo/dupage-county-il#race_and_ethnicity
    let mut hispanic_race_weights: HashMap<Race, f64> = HashMap::from([
        (Race::WhiteCaucasian , 0.0),
        (Race::BlackAfrican   , 0.0),
        (Race::NativeAmerican , 0.0),
        (Race::Asian          , 0.0),
        (Race::PacificIslander, 0.0),
        (Race::Multiracial    , 0.0),
        (Race::Other          , 0.0)
    ]);

    let mut race_weights: HashMap<Race, f64> = HashMap::from([
        (Race::WhiteCaucasian , 0.0),
        (Race::BlackAfrican   , 0.0),
        (Race::NativeAmerican , 0.0),
        (Race::Asian          , 0.0),
        (Race::PacificIslander, 0.0),
        (Race::Multiracial    , 0.0),
        (Race::Hispanic       , 0.0),
        (Race::Other          , 0.0)
    ]);
    
    // Source: https://www.census.gov/library/visualizations/interactive/exploring-age-groups-in-the-2020-census.html
    let mut demographics_reader = csv::Reader::from_path("demographics.csv")?;
    let demographics = demographics_reader
        .deserialize()
        .filter_map::<CountyDemographicsCSVRecord, _>(Result::ok);

    for record in demographics {
        race_weights
            .entry(Race::WhiteCaucasian)
            .and_modify(|count| *count += record.white_alone as f64)
            .or_insert(record.white_alone as f64);
        race_weights
            .entry(Race::BlackAfrican)
            .and_modify(|count| *count += record.black_african_american_alone as f64)
            .or_insert(record.black_african_american_alone as f64);
        race_weights
            .entry(Race::NativeAmerican)
            .and_modify(|count| *count += record.american_indian_alaska_native_alone as f64)
            .or_insert(record.american_indian_alaska_native_alone as f64);
        race_weights
            .entry(Race::Asian)
            .and_modify(|count| *count += record.asian_alone as f64)
            .or_insert(record.asian_alone as f64);
        race_weights
            .entry(Race::PacificIslander)
            .and_modify(|count| *count += record.native_hawaiian_pacific_islander_alone as f64)
            .or_insert(record.native_hawaiian_pacific_islander_alone as f64);
        race_weights
            .entry(Race::Multiracial)
            .and_modify(|count| *count += record.two_or_more_races as f64)
            .or_insert(record.two_or_more_races as f64);
        race_weights
            .entry(Race::Hispanic)
            .and_modify(|count| *count += record.hispanic_latino as f64)
            .or_insert(record.hispanic_latino as f64);
        race_weights
            .entry(Race::Other)
            .and_modify(|count| *count += record.some_other_race_alone as f64)
            .or_insert(record.some_other_race_alone as f64);
    }

    let race_total_population = race_weights.values().sum::<f64>();
    race_weights.values_mut().for_each(|weight| *weight /= race_total_population);

    println!("Race Weights: {:#?}", race_weights);

    let mut hispanic_demographics_reader = csv::Reader::from_path("hispanic_demographics.csv")?;
    let hispanic_demographics = hispanic_demographics_reader
        .deserialize()
        .filter_map::<CountyHispanicDemographicsCSVRecord, _>(Result::ok);

    for record in hispanic_demographics {
        hispanic_race_weights
            .entry(Race::WhiteCaucasian)
            .and_modify(|count| *count += record.white_hispanic as f64)
            .or_insert(record.white_hispanic as f64);
        hispanic_race_weights
            .entry(Race::BlackAfrican)
            .and_modify(|count| *count += record.black_african_american_hispanic as f64)
            .or_insert(record.black_african_american_hispanic as f64);
        hispanic_race_weights
            .entry(Race::NativeAmerican)
            .and_modify(|count| *count += record.american_indian_alaska_native_hispanic as f64)
            .or_insert(record.american_indian_alaska_native_hispanic as f64);
        hispanic_race_weights
            .entry(Race::Asian)
            .and_modify(|count| *count += record.asian_hispanic as f64)
            .or_insert(record.asian_hispanic as f64);
        hispanic_race_weights
            .entry(Race::PacificIslander)
            .and_modify(|count| *count += record.native_hawaiian_pacific_islander_hispanic as f64)
            .or_insert(record.native_hawaiian_pacific_islander_hispanic as f64);
        hispanic_race_weights
            .entry(Race::Multiracial)
            .and_modify(|count| *count += record.two_or_more_races_hispanic as f64)
            .or_insert(record.two_or_more_races_hispanic as f64);
        hispanic_race_weights
            .entry(Race::Other)
            .and_modify(|count| *count += record.some_other_race_hispanic as f64)
            .or_insert(record.some_other_race_hispanic as f64);
    }

    let hispanic_race_total_population = hispanic_race_weights.values().sum::<f64>();
    hispanic_race_weights.values_mut().for_each(|weight| *weight /= hispanic_race_total_population);

    println!("Hispanic Race Weights: {:#?}", hispanic_race_weights);

    let mut reader = csv::Reader::from_path("matches.csv")?;

    // Config
    // Cut-off = 2 to trim sparse samples
    // Cut-off = 0 to include all samples
    const SAMPLE_CUTOFF: u32 = 2;
    let profiles = reader
        .deserialize()
        .enumerate()
        .map(|(i, record)| {
            let line_number = i + 2;
            if let Err(err) = &record {
                println!("error reading record on line {}: {}", line_number, err);
            }

            record.map(|record| (line_number, record))
        })
        .filter_map(Result::ok)
        .filter_map(|(line_number, record): (_, HingeProfileCSVRecord)| {
            let profile = HingeProfile::try_from(record);

            if let Err(err) = &profile {
                println!("error converting record on line {} to profile: {}", line_number, err);
            }

            profile.ok()
        })
        // ! FILTERS GO HERE
        //.filter(|profile| profile.who_last_replied == WhoLastReplied::Met)
        //.filter(|profile| profile.convo)
        //.filter(|profile| profile.ethnicity_specified)
        .collect::<Vec<_>>();

    let mut race_counts = Race::aggregate(profiles.iter().filter_map(|profile| profile.race));
    let mut hispanic_race_counts = Race::aggregate(
        profiles
            .iter()
            .filter(|profile| profile.ethnicity.bits() & Ethnicities::HISPANIC_LATINO != 0)
            .map(|profile| if profile.ethnicity.bits() == Ethnicities::HISPANIC_LATINO 
                { Ethnicities::OTHER } else 
                { profile.ethnicity.bits() & !Ethnicities::HISPANIC_LATINO })
            .filter_map(|ethnicity_bits| Race::try_from(ethnicity_bits).ok()));
    
    

    let total_profiles_with_bkg_info = profiles
        .iter()
        .filter(|profile| profile.race.is_some() || profile.ethnicity.bits() & Ethnicities::HISPANIC_LATINO != 0)
        .count();
    let total_profiles = profiles.len();

    println!("Total Profiles: {}", total_profiles);
    println!("Total Profiles with Background Info: {}", total_profiles_with_bkg_info);
    println!("Race Counts: {:?}", race_counts);
    println!("Hispanic Race Counts: {:?}", hispanic_race_counts);

    race_counts.remove(&Race::Hispanic);
    hispanic_race_counts.remove(&Race::Hispanic);

    println!("Race Index");
    for (race, count) in race_counts.iter() {
        if *count < SAMPLE_CUTOFF {
            println!("\t{:?}: {}", race, "NOT ENOUGH SAMPLES");
        } else {
            println!("\t{:?}: {}", race, *count as f64 / race_weights[race]);
        }
    }
    
    println!("Hispanic Race Index");
    for (race, count) in hispanic_race_counts.iter() {
        if *count < SAMPLE_CUTOFF {
            println!("\t{:?}Hispanic: {}", race, "NOT ENOUGH SAMPLES");
        } else {
            println!("\t{:?}Hispanic: {}", race, *count as f64 / (race_weights[&Race::Hispanic] * hispanic_race_weights[race]));
        }
    }

    Ok(())
}

fn main() {
    if let Err(err) = example() {
        println!("error running example: {}", err);
        process::exit(1);
    }
}