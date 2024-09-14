LET $cpp_id = rand::ulid();
LET $lu_id = rand::ulid();

CREATE university SET
	id = $cpp_id,
	name = "Cal Poly Pomona",
	location.country = "US",
	location.subdivision = "CA",
	location.coordinates = (34.056389, -117.821667)
;

CREATE university SET
	id = $lu_id,
	name = "Lancaster University",
	location.country = "GB",
	location.subdivision = "LAN",
	location.coordinates = (54.010278, -2.785556)
;

LET $jacob_id = rand::ulid();
LET $tigi_id = rand::ulid();

CREATE user SET
	id = $jacob_id,
	name.first = 'Jacob',
	name.last = 'Henn',
	username = 'jacobhenn',
	university = type::thing("university", $cpp_id),
	grad_year = 2026
;

CREATE user SET
	id = $tigi_id,
	name.first = 'Tigius',
	name.last = 'Chubius',
	username = 'tigi',
	university = type::thing("university", $lu_id),
	grad_year = 2026 // ?
;
