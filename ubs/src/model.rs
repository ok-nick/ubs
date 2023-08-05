use chrono::NaiveDate;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ClassSchedule {
    pub groups: Vec<ClassGroup>,
}

#[derive(Debug, Serialize)]
pub struct ClassGroup {
    pub is_open: Option<bool>,
    pub session: Option<u32>,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub classes: Vec<Class>,
}

#[derive(Debug, Serialize)]
pub struct Class {
    // TODO: use ubs_lib::ClassType?
    pub class_type: Option<String>,
    pub class_id: Option<u32>,
    pub section: Option<String>,
    pub days_of_week: Option<Vec<Option<String>>>,
    pub room: Option<String>,
    pub instructor: Option<String>,
    pub open_seats: Option<u32>,
    pub total_seats: Option<u32>,
}

impl TryFrom<ubs_lib::ClassSchedule> for ClassSchedule {
    type Error = ubs_lib::ParseError;

    fn try_from(schedule: ubs_lib::ClassSchedule) -> Result<Self, Self::Error> {
        let mut groups = Vec::new();
        for group in schedule.group_iter() {
            groups.push(ClassGroup::try_from(group)?);
        }

        Ok(ClassSchedule { groups })
    }
}

impl TryFrom<ubs_lib::ClassGroup<'_>> for ClassGroup {
    type Error = ubs_lib::ParseError;

    fn try_from(group: ubs_lib::ClassGroup<'_>) -> Result<Self, Self::Error> {
        let mut classes = Vec::new();
        for class in group.class_iter() {
            classes.push(Class::try_from(class)?);
        }

        Ok(ClassGroup {
            is_open: None, // TODO: this
            session: group.session().ok(),
            start_date: group.start_date().ok(),
            end_date: group.end_date().ok(),
            classes,
        })
    }
}

impl TryFrom<ubs_lib::Class<'_>> for Class {
    type Error = ubs_lib::ParseError;

    fn try_from(class: ubs_lib::Class<'_>) -> Result<Self, Self::Error> {
        Ok(Class {
            class_type: class
                .class_type()
                .ok()
                .map(|class_type| class_type.to_string()),
            class_id: class.class_id().ok(),
            section: class.section().ok().map(ToOwned::to_owned),
            days_of_week: class.days_of_week().ok().flatten().map(|dow| {
                dow.iter()
                    .map(|dow| match dow {
                        Ok(dow) => Some(ToString::to_string(dow)),
                        Err(_) => None,
                    })
                    .collect()
            }),
            room: class.room().ok().map(ToOwned::to_owned),
            instructor: class.instructor().ok().map(ToOwned::to_owned),
            open_seats: class.open_seats().ok().flatten(),
            total_seats: class.total_seats().ok().flatten(),
        })
    }
}
