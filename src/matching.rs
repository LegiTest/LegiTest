// reimplementation of the client-side matching logic
// for consistency checking purposes

use crate::config::structs::{InstanceInfo, Platform};
use crate::database::structs::Choice;
use std::convert::TryFrom;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserOpinion {
    // question_id must be matching a scrutin.question_id
    pub question_id: i16,
    // vote is "pour", "contre" or "abstention"
    pub vote: Choice,
}

#[derive(Debug)]
pub struct GroupScore {
    // id is group_id
    pub id: i16,
    // score is the result of calculateScore
    pub score: i16,
}

#[derive(Debug)]
pub struct GroupMatch {
    // id is group_id
    pub id: i16,
    // affinity is a percent version of GroupScore.score
    // (score + q_count) / (q_count * 2) * 100
    pub affinity: f32,
}

// this function is a copy of client-side's calculateScore
// plus some extra input checking.
// returns None if it can't parse the user's answer
pub fn calculate_score(
    platform: &Platform,
    user_submission: &[UserOpinion],
) -> Option<Vec<GroupScore>> {
    let g_instance = InstanceInfo::global();

    let mut actors: Vec<GroupScore> = Vec::new();

    // list of checked scrutins so we know if someone
    // is trying to send multiple answers to the same scrutin
    let mut checked_scrutins: Vec<i16> = Vec::new();

    // initialize the actors array with the predefined groups list
    for group in &platform.groups {
        actors.push(GroupScore {
            id: *group,
            score: 0,
        });
    }

    // check: number of opinions == number of scrutins
    if user_submission.len() != g_instance.scrutins_list.len() {
        eprintln!(
            "warn: haxx0r tried to submit {} out of {} opinions.",
            user_submission.len(),
            g_instance.scrutins_list.len()
        );
        return None;
    }

    for opinion in user_submission {
        // check for already submitted scrutins with the same ID
        if checked_scrutins
            .iter()
            .any(|scr| scr == &opinion.question_id)
        {
            eprintln!("warn: haxx0r tried to submit multiple answers for the same scrutin.");
            return None;
        }
        checked_scrutins.push(opinion.question_id);

        // get the corresponding scrutin
        // NOTE: will cause bugs on reports generation
        // if questions are removed afterwards
        let scrutin = g_instance
            .scrutins_list
            .iter()
            .find(|scr| scr.question_id == opinion.question_id)?;

        // for scrutin with inverted votes, invert the default score
        let default_score = if scrutin.invert_votes { -1 } else { 1 };

        for (gr_choice, gr_ids) in &scrutin.organes {
            let gr_score = match (&opinion.vote, gr_choice) {
                (Choice::Pour, Choice::Pour) | (Choice::Contre, Choice::Contre) => default_score,
                (Choice::Pour, Choice::Contre) | (Choice::Contre, Choice::Pour) => -default_score,
                (Choice::Pour | Choice::Contre, Choice::Abstention)
                | (Choice::Abstention, Choice::Pour | Choice::Contre) => 0,
                (Choice::Abstention, Choice::Abstention) => 1,
            };

            // find corresponding actors and save their scores
            for gr_id in gr_ids {
                for act in &mut actors {
                    if act.id == *gr_id {
                        act.score += gr_score;
                        break;
                    }
                }
            }
        }
    }
    Some(actors)
}

// questions count is used to not fail the score calculation if a
// question is removed halfway
pub fn calculate_affinity(group_scores: &[GroupScore], qcount: usize) -> Option<Vec<GroupMatch>> {
    let mut group_match: Vec<GroupMatch> = Vec::new();
    for score in group_scores {
        group_match.push(GroupMatch {
            id: score.id,
            affinity: (f32::try_from(score.score + i16::try_from(qcount).ok()?).ok()?
                / f32::try_from(i16::try_from(qcount).ok()? * 2).ok()?)
                * 100_f32,
        });
    }
    Some(group_match)
}
