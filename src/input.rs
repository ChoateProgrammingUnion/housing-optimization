extern crate yaml_rust;
use self::yaml_rust::YamlLoader;

use crate::ballot::{Ballot, House, Student};

use std::collections::HashMap;
use std::io::Read;

pub fn load_trials() -> Vec<String> {
    let mut trials = Vec::<String>::new();
    let mut input_str: String = String::new();

    let mut trial_file = std::fs::File::open("config.yaml").expect("yaml file not found");
    trial_file
        .read_to_string(&mut input_str)
        .expect("input file read failed");
    let trial_runs = YamlLoader::load_from_str(&*input_str).expect("yaml failed to load");


    for trial in trial_runs[0]["run"].clone() {
        crate::log_debug!(format!("trial {} ran", trial.as_str().unwrap()), "input");
        trials.push(trial.into_string().unwrap());
    }

    return trials;
}

pub fn load_input(process: fn(Student) -> Student) -> Ballot {
    // Load file
    let mut input_file = std::fs::File::open("input.yaml").expect("yaml file not found");

    // Read file
    let mut input_str: String = String::new();
    input_file
        .read_to_string(&mut input_str)
        .expect("input file read failed");
    let input = YamlLoader::load_from_str(&*input_str).expect("yaml failed to load");

    // Load input
    let houses = input[0]["houses"]
        .clone()
        .into_vec()
        .expect("houses is not an array");
    let ballots = input[0]["ballots"]
        .clone()
        .into_vec()
        .expect("ballots is not an array");
    crate::log_debug!(
        format!(
            "yaml loaded to arrays ({} houses, {} students)",
            houses.len(),
            ballots.len()
        ),
        "input"
    );

    // Hash map for changing house name into id
    let mut house_name_map: HashMap<String, f64> = HashMap::new();

    // Hash map for changing friend name into id
    let mut student_name_map: HashMap<String, usize> = HashMap::new();

    // Set up Ballot object
    let mut new_ballot = Ballot::new();
    let num_houses = houses.len();

    // Add houses
    for house in houses {
        let name = house["name"].as_str().expect("house name is not a string");
        let capacity = house["capacity"]
            .as_i64()
            .expect("house capacity is not an integer");

        let new_house = House::new(String::from(name), capacity as usize);

        crate::log_trace!(format!("adding house to list: {:?}", new_house), "input");

        house_name_map.insert(String::from(name), new_ballot.houses.len() as f64);
        new_ballot.houses.push(new_house)
    }

    // Add ballots
    for ballot in &ballots {
        let student_name = ballot["name"]
            .as_str()
            .expect("student name is not a string");
        let rankings = ballot["ranking"]
            .clone()
            .into_vec()
            .expect("student rankings is not an array");

        let mut student = Student::new(
            String::from(student_name),
            num_houses,
            new_ballot.students.len(),
        );

        student_name_map.insert(student.name.clone(), student.id);

        for ranking in rankings {
            let house_name = ranking["name"]
                .as_str()
                .expect("house name is not a string");
            let house_weight = ranking["weight"]
                .as_f64()
                .expect("house weight is not a float");
            let house_index = house_name_map[house_name];
            student.ballot[house_index as usize] = house_weight;
        }

        crate::log_trace!(
            format!(
                "preprocessed student: {}({:?})",
                student.name, student.ballot
            ),
            "input"
        );

        let mut processed_student = process(student);
        for n in &processed_student.ballot {
            processed_student.ballot_sum += n;
        }

        crate::log_trace!(
            format!(
                "processed ballot: {}({:?}, sum={})",
                processed_student.name, processed_student.ballot, processed_student.ballot_sum
            ),
            "input"
        );

        new_ballot.students.push(processed_student);
    }

    // Add friends
    for ballot in &ballots {
        let student_name = ballot["name"]
            .as_str()
            .expect("student name is not a string");
        let friends = ballot["friends"].clone().into_vec().unwrap_or(vec![]);

        let student_index = student_name_map[student_name];

        for friend in friends {
            let friend_name = friend.as_str().expect("friend name is not string");
            let friend_id = student_name_map[friend_name];
            new_ballot.students[student_index].friends.push(friend_id);
        }
    }

    new_ballot
}
