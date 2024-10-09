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
    fn entries() -> impl Iterator<Item = Self> {
        [
            Race::WhiteCaucasian, 
            Race::BlackAfrican, 
            Race::NativeAmerican, 
            Race::Asian, 
            Race::PacificIslander, 
            Race::Multiracial, 
            Race::Hispanic, 
            Race::Other
        ].iter().copied()
    }

    fn aggregate(races: impl Iterator<Item = Self>) -> HashMap<Self, u32> {
        let mut race_counts = HashMap::from_iter(Race::entries().map(|race| (race, 0)));
        for race in races {
            race_counts.entry(race).and_modify(|count| *count += 1).or_insert(1);
        }
    
        race_counts
    }
}

impl std::fmt::Display for Race {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match *self {
            Race::WhiteCaucasian => "White",
            Race::BlackAfrican => "Black or African American",
            Race::NativeAmerican => "American Indian or Alaska Native",
            Race::Asian => "Asian",
            Race::PacificIslander => "Native Hawaiian & Other Pacific Island",
            Race::Multiracial => "Multiracial",
            Race::Hispanic => "Hispanic or Latino",
            Race::Other => "Other"
        })?;

        Ok(())
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

        if who_last_replied == WhoLastReplied::Met && value.convo == 0 {
            return Err("Who Last Replied is Met but Conversation is False");
        }

        if who_last_replied == WhoLastReplied::None && value.convo != 0 {
            return Err("Who Last Replied is None but Conversation is True");
        }

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

#[derive(Debug)]
struct RacialPreference {
    race: Race,
    hispanic: bool,
    weight: f64,
    count: u32
}

impl std::fmt::Display for RacialPreference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:<55}   {:.4}   {}", format!("{} ({})", self.race, if self.hispanic { "Hispanic" } else { "Non-Hispanic" }), self.weight, self.count)?;
        Ok(())
    }
}

// fn aggregate_racial_preferences(
//     profiles: &[HingeProfile], 
//     race_distribution: HashMap<Race, f64>,
//     race_distribution_hispanic: HashMap<Race, f64>
// ) -> Vec<RacialPreference> {

//     let mut race_counts = HashMap::from_iter(Race::entries().map(|race| (race, 0)));

//     let races = profiles.iter().filter_map(|profile| profile.race);
//     let hispanic_races = profiles
//         .iter()
//         .filter(|profile| profile.ethnicity.bits() & Ethnicities::HISPANIC_LATINO != 0)
//         .map(|profile| if profile.ethnicity.bits() == Ethnicities::HISPANIC_LATINO 
//             { Ethnicities::OTHER } else 
//             { profile.ethnicity.bits() & !Ethnicities::HISPANIC_LATINO })
//         .filter_map(|ethnicity_bits| Race::try_from(ethnicity_bits).ok());

    
    
    

//     let total_profiles_with_bkg_info = profiles
//         .iter()
//         .filter(|profile| profile.race.is_some() || profile.ethnicity.bits() & Ethnicities::HISPANIC_LATINO != 0)
//         .count();
//     let total_profiles = profiles.len();

//     hispanic_race_counts.remove(&Race::Hispanic);

//     println!("Total Profiles: {}", total_profiles);
//     println!("Total Profiles with Background Info: {}", total_profiles_with_bkg_info);
//     println!("Race Counts: {:?}", race_counts);
//     println!("Race (Hispanic) Breakdown: {:?}", hispanic_race_counts);

//     race_counts.remove(&Race::Hispanic);

//     println!("Race Index");
//     for (race, count) in race_counts.iter() {
//         if *count < SAMPLE_CUTOFF {
//             println!("\t{:?}: {}", race, "NOT ENOUGH SAMPLES");
//         } else {
//             println!("\t{:?}: {}", race, *count as f64 / race_weights[race]);
//         }
//     }
    
//     println!("Hispanic Race Index");
//     for (race, count) in hispanic_race_counts.iter() {
//         if *count < SAMPLE_CUTOFF {
//             println!("\t{:?}Hispanic: {}", race, "NOT ENOUGH SAMPLES");
//         } else {
//             println!("\t{:?}Hispanic: {}", race, *count as f64 / (race_weights[&Race::Hispanic] * hispanic_race_weights[race]));
//         }
//     }
// }

fn run_analysis() -> Result<(), Box<dyn Error>> {

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

    hispanic_race_counts.remove(&Race::Hispanic);

    println!("\nTotal Profiles: {}", total_profiles);
    println!("Total Profiles with Background Info: {}", total_profiles_with_bkg_info);
    //println!("\nRace Counts: {:?}", race_counts);
    //println!("Race (Hispanic) Breakdown: {:?}", hispanic_race_counts);

    race_counts.remove(&Race::Hispanic);

    let mut racial_preferences = Vec::new();

    for (&race, &count) in race_counts.iter() {
        racial_preferences.push(RacialPreference { 
            race, 
            hispanic: false, 
            weight: if count < SAMPLE_CUTOFF { 0.0 } else { count as f64 / race_weights[&race] },
            count
        });
    }

    for (&race, &count) in hispanic_race_counts.iter() {
        racial_preferences.push(RacialPreference { 
            race, 
            hispanic: true, 
            weight: if count < SAMPLE_CUTOFF { 0.0 } else { count as f64 / (race_weights[&Race::Hispanic] * hispanic_race_weights[&race]) },
            count
        });
    }

    let racial_preferences_total_weight = racial_preferences.iter().map(|preference| preference.weight).sum::<f64>();
    racial_preferences.iter_mut().for_each(|preference| preference.weight /= racial_preferences_total_weight);
    racial_preferences.sort_by(|a, b| b.weight.partial_cmp(&a.weight).expect("Bad comparison in racial preferences"));

    println!("\n\t  Race Preference Index (Adjusted for Population, Match Sample Cutoff={})", SAMPLE_CUTOFF);
    println!("\t{:^55}   {}   {}", "Race", "Weight", "Matches"); 
    for preference in racial_preferences.iter() {
        println!("\t{}", preference);
    }

    // Metrics
    let mut no_convo_count = 0;
    let mut no_convo_attempted_count = 0;
    let mut no_convo_you_failed_count = 0;
    let mut no_convo_they_failed_count = 0;
    let mut convo_started_count = 0;
    let mut convo_started_you_failed_count = 0;
    let mut convo_started_they_failed_count = 0;
    let mut you_met_count = 0;

    for profile in profiles.iter() {
        if profile.convo {
            convo_started_count += 1;
            match profile.who_last_replied {
                WhoLastReplied::You => convo_started_you_failed_count += 1,
                WhoLastReplied::Them => convo_started_they_failed_count += 1,
                WhoLastReplied::Met => you_met_count += 1,
                WhoLastReplied::None => unreachable!("None should not be in convo")
            }
        } else {
            no_convo_count += 1;
            match profile.who_last_replied {
                WhoLastReplied::You => no_convo_you_failed_count += 1,
                WhoLastReplied::Them => no_convo_they_failed_count += 1,
                WhoLastReplied::None => no_convo_attempted_count += 1,
                WhoLastReplied::Met => unreachable!("Met should not be in no convo")
            }
        }
    }

    let convo_you_attempted_count = total_profiles - no_convo_attempted_count - no_convo_they_failed_count;
    
    let conversation_interested_score = convo_you_attempted_count as f64 / total_profiles as f64;
    let conversation_they_failed_score = no_convo_they_failed_count as f64 / total_profiles as f64;
    let conversation_no_one_interested_score = no_convo_attempted_count as f64 / total_profiles as f64;
    let conversation_starter_score = convo_started_count as f64 / convo_you_attempted_count as f64;
    let conversation_starter_failed_score = no_convo_you_failed_count as f64 / convo_you_attempted_count as f64;
    let conversation_to_them_ghosting_score = convo_started_you_failed_count as f64 / convo_started_count as f64;
    let conversation_to_you_ghosting_score = convo_started_they_failed_count as f64 / convo_started_count as f64;
    let conversation_to_date_score = you_met_count as f64 / convo_started_count as f64;

    println!("\nGhosting Metrics");
    println!("You end up ghosting {:.2}% of your matches, {:.2}% of your matches end up ghosting you, {:.2}% of your matches have no activity, and {:.2}% of your matches result in a date.", 
        (no_convo_they_failed_count + convo_started_they_failed_count) as f64 / total_profiles as f64 * 100.0, 
        (no_convo_you_failed_count + convo_started_you_failed_count) as f64 / total_profiles as f64 * 100.0,
        no_convo_attempted_count as f64 / total_profiles as f64 * 100.0,
        you_met_count as f64 / total_profiles as f64 * 100.0);

    
    println!("\nConversation Success Metrics");
    println!("You are interested in having a conversation with {:.2}% of your matches, {:.2}% of the time you are not interested despite receiving a message, {:.2}% of the time no one is interested.", 
        conversation_interested_score * 100.0,
        conversation_they_failed_score * 100.0,
        conversation_no_one_interested_score * 100.0
    );
    println!("Of those you are interested in having a conversation with, you succeed {:.2}% of the time and fail {:.2}% of the time.", 
        conversation_starter_score * 100.0,
        conversation_starter_failed_score * 100.0);
    println!("Of those you succeed in starting a conversation with, you eventually ghost them {:.2}% of the time, they eventually ghost you {:.2}% of the time, and you go on a date with {:.2}% of them.", 
        conversation_to_you_ghosting_score * 100.0, 
        conversation_to_them_ghosting_score * 100.0, 
        conversation_to_date_score * 100.0);
    
    println!("\nDate Conversion Rate");
    println!("Given that you're interested in having a conversation with your match, there's a {:.2}% chance you go on a date.", 
        conversation_starter_score * conversation_to_date_score * 100.0);

    Ok(())
}

fn main() {
    if let Err(err) = run_analysis() {
        println!("error running example: {}", err);
        process::exit(1);
    }
}