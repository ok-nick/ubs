//! Models of parser structs with all fields evaluated.

use chrono::{NaiveDate, NaiveTime};
#[cfg(feature = "serde_support")]
use serde::{Deserialize, Serialize};

use crate::parser::{Class, ClassGroup, ClassSchedule, ClassType, DayOfWeek, ParseError};

// TODO: document models

/// Model of a [`ClassSchedule`](ClassSchedule) with all fields evaluated.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
pub struct ClassScheduleModel {
    // TODO: add semester?
    pub groups: Vec<ClassGroupModel>,
}

/// Model of a [`ClassGroup`](ClassGroup) with all fields evaluated.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
pub struct ClassGroupModel {
    pub session: Option<u32>,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub classes: Vec<ClassModel>,
}

/// Model of a [`Class`](Class) with all fields evaluated.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
pub struct ClassModel {
    pub is_open: Option<bool>,
    pub class_type: Option<ClassType>,
    pub class_id: Option<u32>,
    pub section: Option<String>,
    pub days_of_week: Option<Vec<Option<DayOfWeek>>>,
    pub start_time: Option<NaiveTime>,
    pub end_time: Option<NaiveTime>,
    pub room: Option<String>,
    pub instructor: Option<String>,
    pub open_seats: Option<u32>,
    pub total_seats: Option<u32>,
}

impl TryFrom<&ClassSchedule> for ClassScheduleModel {
    type Error = ParseError;

    fn try_from(schedule: &ClassSchedule) -> Result<Self, Self::Error> {
        let mut groups = Vec::new();
        for group in schedule.group_iter() {
            groups.push(ClassGroupModel::try_from(&group)?);
        }

        Ok(ClassScheduleModel { groups })
    }
}

impl TryFrom<&ClassGroup<'_>> for ClassGroupModel {
    type Error = ParseError;

    fn try_from(group: &ClassGroup<'_>) -> Result<Self, Self::Error> {
        let mut classes = Vec::new();
        for class in group.class_iter() {
            classes.push(ClassModel::try_from(&class)?);
        }

        Ok(ClassGroupModel {
            session: group.session().ok(),
            start_date: group.start_date().ok(),
            end_date: group.end_date().ok(),
            classes,
        })
    }
}

impl TryFrom<&Class<'_>> for ClassModel {
    type Error = ParseError;

    fn try_from(class: &Class<'_>) -> Result<Self, Self::Error> {
        Ok(ClassModel {
            is_open: class.is_open().ok(),
            class_type: class.class_type().ok(),
            class_id: class.class_id().ok(),
            section: class.section().ok().map(ToOwned::to_owned),
            days_of_week: class
                .days_of_week()
                .ok()
                .flatten()
                .map(|dow| dow.into_iter().map(|dow| dow.ok()).collect()),
            start_time: class.start_time()?,
            end_time: class.end_time()?,
            room: class.room().ok().map(ToOwned::to_owned),
            instructor: class.instructor().ok().map(ToOwned::to_owned),
            open_seats: class.open_seats().ok().flatten(),
            total_seats: class.total_seats().ok().flatten(),
        })
    }
}
