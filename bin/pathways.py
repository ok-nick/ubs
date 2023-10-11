import json
import csv
import sys

# all pathways are undergraduate
CAREER = "UGRD"

# so that we can add onto the csv of courses without adding duplicates
exists = set()
try:
    # the final output should be in `ubs-lib/data/courses.csv`
    with open(sys.argv[2]) as output:
        reader = csv.reader(output)
        for row in reader:
            exists.add(row[1])
except IOError:
    pass


# input requires authorization, thus manual input until a better API is found
# INPUT: https://path-finder.apps.buffalo.edu/api/cached/topics
with open(sys.argv[1]) as input, open(sys.argv[2], "a+") as output:
    pathways = json.load(input)

    courses = []
    for group in pathways["data"]:
        for courses_info in group["courses"].values():
            for course_info in courses_info:
                if course_info["course_id"] not in exists:
                    exists.add(course_info["course_id"])

                    courses.append(
                        [
                            course_info["course_id"],
                            CAREER,
                            course_info["subject"] + course_info["catalog_number"],
                        ]
                    )

    writer = csv.writer(output)
    writer.writerows(courses)
