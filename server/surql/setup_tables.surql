// specify the types of these fields as `set` so they are automatically deduplicated
DEFINE FIELD assignments_planning ON TABLE user TYPE set<record<assignment>>;
DEFINE FIELD assignments_in_progress ON TABLE user TYPE set<record<assignment>>;
DEFINE FIELD assignments_completed ON TABLE user TYPE set<record<assignment>>;

// --------------------------------------------------------
// whenever an activity is registered, update the relevant user data
DEFINE EVENT update_users ON TABLE activity
	WHEN $before = NONE
	THEN {
		IF $after.data.kind = 'Planning' {
			UPDATE $after.user SET assignments_planning += $after.assignment;
		} ELSE IF $after.data.kind = 'WorkedOn' {
			// move assignment from planning to in progress. since these fields are sets, this only
			// does something if it needs to
			UPDATE $after.user SET
				assignments_planning -= $after.assignment,
				assignments_in_progress += $after.assignment
			;

			UPDATE $after.user SET stats.secs_worked += $after.data.duration_secs;
		} ELSE IF $after.data.kind = 'Completed' {
			UPDATE $after.user SET
				assignments_in_progress -= $after.assignment,
				assignments_completed += $after.assignment
			;
		
			UPDATE $after.user SET stats.assignments_completed += 1;
		};
	};

