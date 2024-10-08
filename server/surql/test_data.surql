// --------------------------------------------------------
// Universities

LET $cpp = (CREATE ONLY university:ulid() SET
	name = "Cal Poly Pomona",
	location.country = "US",
	location.subdivision = "CA",
	location.coordinates = (34.056389, -117.821667)
).id;

LET $lu = (CREATE ONLY university:ulid() SET
	name = "Lancaster University",
	location.country = "GB",
	location.subdivision = "LAN",
	location.coordinates = (54.010278, -2.785556)
).id;

// --------------------------------------------------------
// Majors

LET $math = (CREATE ONLY major:ulid() SET name = "Mathematics").id;
LET $cs = (CREATE ONLY major:ulid() SET name = "Computer Science").id;

// --------------------------------------------------------
// Courses
//
// These would not be automatically populated in production, but input by users

LET $cpp_mat4170 = (CREATE ONLY course:ulid() SET
	university = $cpp,
	code = "MAT 4170",
	name = "Abstract Algebra I"
).id;

LET $cpp_sta2100 = (CREATE ONLY course:ulid() SET
	university = $cpp,
	code = "STA 2100",
	name = "Intro to Statistics"
).id;

LET $cpp_phl2040 = (CREATE ONLY course:ulid() SET
	university = $cpp,
	code = "PHL 2040",
	name = "Ethical Problems in Contemporary Life"
).id;

LET $cpp_bio1110 = (CREATE ONLY course:ulid() SET
	university = $cpp,
	code = "BIO 1110",
	name = "Intro to Life Science"
).id;

LET $cpp_cs3750 = (CREATE ONLY course:ulid() SET
	university = $cpp,
	code = "CS 3750",
	name = "Computers and Society"
).id;

LET $cpp_cs2410 = (CREATE ONLY course:ulid() SET
	university = $cpp,
	code = "CS 2410",
	name = "Fundamentals of Data Science"
).id;

LET $cpp_cs2600 = (CREATE ONLY course:ulid() SET
	university = $cpp,
	code = "CS 2600",
	name = "Systems Programming"
).id;

// --------------------------------------------------------
// Assignments

LET $cpp_mat4170_hw2 = (CREATE ONLY assignment:ulid() SET
	course = $cpp_mat4170,
	name = "HW 2"
).id;

LET $cpp_cs2600_quiz1 = (CREATE ONLY assignment:ulid() SET
	course = $cpp_cs2600,
	name = "Quiz 1"
).id;

// --------------------------------------------------------
// Users

LET $jacob = (CREATE ONLY user:ulid() SET
	name.first = 'Jacob',
	name.last = 'Henn',
	username = 'jacobhenn',
	profile_picture = '01J8ZTZKNXF4TCQ6AC0P61FBKW',
	university = $cpp,
	major = $math,
	grad_year = 2026,
	assignments_planning = [],
	assignments_in_progress = [],
	assignments_completed = [],
	stats.num_assignments_completed = 0,
	stats.secs_worked = 0
).id;

LET $amy = (CREATE ONLY user:ulid() SET
	name.first = 'Amy',
	name.last = 'Nguyen',
	username = 'choobipanda',
	university = $cpp,
	major = $cs,
	grad_year = 2026,
	assignments_planning = [],
	assignments_in_progress = [],
	assignments_completed = [],
	stats.num_assignments_completed = 0,
	stats.secs_worked = 0
).id;

LET $tigi = (CREATE ONLY user:ulid() SET
	name.first = 'Tigius',
	name.last = 'Chubius',
	username = 'tigi',
	university = $lu,
	major = $cs,
	grad_year = 2026,
	assignments_planning = [],
	assignments_in_progress = [],
	assignments_completed = [],
	stats.num_assignments_completed = 0,
	stats.secs_worked = 0
).id;

// --------------------------------------------------------
// Course enrollment relationships

RELATE $jacob->takes_course->$cpp_mat4170;
RELATE $jacob->takes_course->$cpp_sta2100;
RELATE $jacob->takes_course->$cpp_phl2040;
RELATE $jacob->takes_course->$cpp_bio1110;

RELATE $amy->takes_course->$cpp_cs3750;
RELATE $amy->takes_course->$cpp_cs2410;
RELATE $amy->takes_course->$cpp_cs2600;
RELATE $amy->takes_course->$cpp_bio1110;

// --------------------------------------------------------
// Follow relationships

RELATE $jacob->follows->$amy;
RELATE $amy->follows->$jacob;
RELATE $amy->follows->$tigi;
RELATE $tigi->follows->$jacob;

// --------------------------------------------------------
// Activity

CREATE activity:ulid() SET
	user = $jacob,
	time = time::now(),
	course = $cpp_mat4170,
	assignment = $cpp_mat4170_hw2,
	data.kind = 'Planning'
;

CREATE activity:ulid() SET
	user = $amy,
	time = time::now(),
	course = $cpp_cs2600,
	assignment = $cpp_cs2600_quiz1,
	data.kind = 'WorkedOn',
	data.duration_secs = duration::secs(25m)
;

CREATE activity:ulid() SET
	user = $amy,
	time = time::now(),
	course = $cpp_cs2600,
	assignment = $cpp_cs2600_quiz1,
	data.kind = 'Completed'
;
