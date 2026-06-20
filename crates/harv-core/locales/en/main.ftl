# Errors
err-not-authenticated = You are not authenticated. Run `harv connect` to log in to your Harvest account.
err-config-not-found = Config file not found. Run `harv connect` to log in to your Harvest account.
err-config-malformed = Config file is malformed: { $msg }
err-http = HTTP error: { $msg }
err-api = Harvest API returned error ({ $status }): { $message }
err-io = IO error: { $msg }
err-invalid-date = Invalid date: { $msg }
err-invalid-time = Invalid time: { $msg }
err-no-running-timer = No timer is currently running.
err-no-project-assignments = You have no project assignments.
err-no-task-assignments = No task assignments found for project { $project_id }.
err-alias-not-found = Alias '{ $name }' not found. Use `harv alias list` to view aliases.
err-oauth-failed = Access token could not be retrieved from the OAuth2 response.
err-oauth-denied = Authorization was denied. Try again with `harv connect`.

# Date/Time
datetime-hours-suffix = h

# Text
text-yes = Yes
text-no = No

# CLI — Auth
cli-auth-manual-url = If the browser does not open, visit the URL shown below.

# CLI — Connect
cli-connect-opening = Opening browser for Harvest authentication...
cli-connect-success = Successfully authenticated with Harvest. Config saved to { $path }.
cli-connect-failed = Authentication failed: { $err }
cli-connect-save-failed = Failed to save config: { $err }

# CLI — Track
cli-track-loading-projects = Loading project assignments...
cli-track-project-not-found = Project ID { $pid } not found in your assignments.
cli-track-task-not-assigned = Task ID { $tid } not assigned to project { $pid }.
cli-track-creating = Creating time entry...
cli-track-timer-started = Timer started! { $confirmation }
cli-track-created = Created: { $confirmation }

# CLI — Start
cli-start-delegates = Delegating to harv track...

# CLI — Stop
cli-stop-loading = Loading...
cli-stop-no-timer = No timer is currently running.
cli-stop-prompt-which = Which timer do you want to stop?
cli-stop-prompt-notes = Notes (empty to keep current):
cli-stop-success = Timer stopped.
cli-stop-detail =   #{ $id }	{ $client } → { $project } → { $task }	{ $hours }h

# CLI — Status
cli-status-loading = Loading...
cli-status-no-timer = No timer is currently running.
cli-status-running-header = Running timers:
cli-status-today-header = Today's entries ({ $total }h total):
cli-status-entry-line =   #{ $id }	{ $hours }	{ $project } → { $task }	{ $notes }

# CLI — Edit
cli-edit-loading = Loading...
cli-edit-loading-details = Loading entry details...
cli-edit-prompt-running = Which timer do you want to edit?
cli-edit-prompt-entry = Which entry do you want to edit?
cli-edit-no-entries = No entries to edit. Use `harv track` to create one.
cli-edit-running-rejects-hours = Cannot change hours on a running timer. Stop it first with `harv stop`.
cli-edit-running-rejects-date = Cannot change the date on a running timer. Stop it first with `harv stop`.
cli-edit-project-not-found = Project ID { $pid } not found in your assignments.
cli-edit-task-not-assigned = Task ID { $tid } not assigned to project { $pid }.
cli-edit-notes-prompt = Notes (current: "{ $existing }", empty to keep):
cli-edit-saving = Saving changes...
cli-edit-success = Updated: #{ $id } — { $hours } — { $date } → { $project } → { $task }
cli-edit-entry-line = #{ $id }  { $hours }  { $project } → { $task }
cli-edit-status-running = Running

# CLI — Note
cli-note-loading = Loading...
cli-note-no-timer = No timer is currently running.
cli-note-prompt-which = Which timer?
cli-note-prompt-notes = Notes (empty to keep current):
cli-note-success = Notes updated for timer #{ $id }.
cli-note-noop = Nothing to update for timer #{ $id }.

# CLI — Projects
cli-projects-loading = Loading project assignments...
cli-projects-header-client = Client
cli-projects-header-project = Project
cli-projects-header-tasks = Tasks
cli-projects-header-id = Project ID

# CLI — Tasks
cli-tasks-header-task = Task
cli-tasks-header-id = Task ID
cli-tasks-header-billable = Billable

# CLI — Whoami
cli-whoami-not-auth = Not authenticated.
cli-whoami-not-auth-hint = Run `harv connect` to log in with your Harvest account.
cli-whoami-warning-company = Warning: could not fetch company info: { $err }
cli-whoami-account-label = Authenticated to account { $account_id }
cli-whoami-name = Name:
cli-whoami-email = Email:
cli-whoami-active = Active:
cli-whoami-timezone = Timezone:
cli-whoami-capacity = Weekly capacity:
cli-whoami-roles = Access roles:
cli-whoami-company = Company:

# CLI — Disconnect
cli-disconnect-not-auth = Not authenticated. Nothing to disconnect.
cli-disconnect-disconnecting = Disconnecting from Harvest account { $id }...
cli-disconnect-disconnecting-no-id = Disconnecting...
cli-disconnect-warning-cache = Warning: could not clear project cache: { $err }
cli-disconnect-removed = Config removed: { $path }
cli-disconnect-cache-cleared = Project cache cleared.
cli-disconnect-done = You are now disconnected. Run `harv connect` to log back in.

# CLI — Config
cli-config-file = Config file: { $path }
cli-config-not-found = (not found)
cli-config-access-token = access-token:
cli-config-account-id = account-id:
cli-config-locale = locale:
cli-config-auto-detect = (auto-detect)
cli-config-cache-ttl = cache-ttl:
cli-config-check-updates = check-updates:
cli-config-aliases = aliases:
cli-config-none-bare = (none)
cli-config-redacted = <redacted>
cli-config-unknown-setting = Unknown setting: { $setting }. Valid settings: access-token, account-id, locale, aliases, cache-ttl, check-updates
cli-config-cache-ttl-invalid = cache-ttl must be a positive number
cli-config-check-updates-invalid = check-updates must be true or false
cli-config-locale-invalid = Invalid locale: { $value }. Supported: { $supported } (or empty/auto to reset)
cli-config-unknown-setting-set = Unknown setting: { $setting }. Valid settings: cache-ttl, locale, check-updates
cli-config-set-success = { $setting } set to { $value }
cli-config-load-failed = Failed to load config: { $err }
cli-config-save-failed = Failed to save config: { $err }
cli-config-alias-format = { $name } -> project: { $pid }, task: { $tid }

# CLI — Alias
cli-alias-loading = Loading project assignments...
cli-alias-no-assignments = No project assignments found.
cli-alias-created = Alias '{ $name }' created: { $display } => { $task }
cli-alias-list-empty = No aliases defined. Use `harv alias create` to create one.
cli-alias-header-alias = Alias
cli-alias-header-project = Project
cli-alias-header-task = Task
cli-alias-not-found = Alias '{ $name }' not found.
cli-alias-deleted = Alias '{ $name }' deleted.

# CLI — Prompts
cli-prompt-alias-name = Alias name:
cli-prompt-alias-empty = Alias name cannot be empty
cli-prompt-alias-spaces = Alias name cannot contain spaces
cli-prompt-project = Project:
cli-prompt-task = Task:
cli-prompt-date = Date:
cli-prompt-date-future = Date cannot be in the future
cli-prompt-date-invalid = Invalid date format (YYYY-MM-DD)
cli-prompt-hours = Hours (0 to start timer, e.g. 1.5 or 1:30):
cli-prompt-hours-negative = Hours must be non-negative
cli-prompt-notes-editor = Notes (opens $EDITOR, empty to skip):
cli-prompt-notes = Notes (empty to skip):
cli-prompt-date-keep = Date (empty to keep current):
cli-prompt-hours-keep = Hours (empty to keep, 0 to clear):
cli-prompt-hours-format-hint = Use HH:MM format (e.g. 1:30)
cli-prompt-hours-invalid-hhmm = Invalid hours in HH:MM
cli-prompt-hours-invalid-minutes = Invalid minutes in HH:MM
cli-prompt-hours-range = Minutes must be 0-59
cli-prompt-hours-non-negative = Hours must be non-negative
cli-prompt-hours-format-error = Enter a valid number or HH:MM format (e.g. 1:30)
cli-prompt-date-future-error = Date { $date } is in the future

# TUI — App
tui-app-title = HARV
tui-app-running = ● Running
tui-app-idle = ○ Idle
tui-app-update = v{ $version } available
tui-app-confirm-title = Confirm
tui-app-confirm-prompt = y = confirm   any other key = cancel
tui-app-confirm-delete = "{ $desc }" Delete this entry?
tui-app-confirm-stop-start = A timer is currently running: "{ $desc }"  Stop it and start a new one?
tui-app-loading-create = Creating entry...
tui-app-loading-save = Saving changes...
tui-app-loading-sync = Refreshing dashboard...
tui-app-loading-entries = Loading time entries...
tui-app-loading-assignments = Loading project data...
tui-app-loading-generic = Loading...
tui-app-loading-stop = Stopping timer...
tui-app-loading-delete = Deleting entry...

# TUI — Dashboard
tui-dash-date-prev = <
tui-dash-date-next = >
tui-dash-today = (Today)
tui-dash-running-header = RUNNING { $elapsed }
tui-dash-idle-header = IDLE
tui-dash-no-timer = No timer running
tui-dash-table-project = Project
tui-dash-table-hours = Hours
tui-dash-table-task = Task
tui-dash-table-notes = Notes
tui-dash-block-today = Today
tui-dash-hours-total = { $total }h total
tui-dash-running-prefix = ●
tui-dash-empty-today = No entries today. Press n to start tracking!
tui-dash-empty-past = No entries for { $date }. Press n to log time!
tui-dash-desc-running = running
tui-dash-desc-stopped = stopped

# TUI — Shortcuts
tui-short-day = Day
tui-short-pick = Pick
tui-short-new = New
tui-short-start = Start
tui-short-edit = Edit
tui-short-stop = Stop
tui-short-del = Del
tui-short-refr = Refr
tui-short-quit = Quit
tui-short-help = Help

# TUI — Form
tui-form-title-start = Start Timer
tui-form-title-create = New Entry
tui-form-title-edit = Edit Entry
tui-form-date-label = Date (YYYY-MM-DD)
tui-form-hours-label = Hours (e.g. 1.5 or 1:30)
tui-form-notes-label = Notes (optional)
tui-form-project-title = Project
tui-form-project-search = Project [{ $search }]
tui-form-task-title = Task
tui-form-task-search = Task [{ $search }]
tui-form-project-loading = Loading projects...
tui-form-project-empty = No project assignments
tui-form-task-empty = No tasks available
tui-form-task-select-first = Select a project first
tui-form-empty-field = (empty)
tui-form-help-create = Tab: next field │ Enter: next/send │ Esc: cancel
tui-form-help-start = Tab: next field │ Enter: start timer │ Esc: cancel
tui-form-help-edit = Tab: next field │ Enter: save │ Esc: cancel

# TUI — Help
tui-help-title = Help
tui-help-section-nav = Navigation
tui-help-nav-down = Move down (lists)
tui-help-nav-up = Move up (lists)
tui-help-nav-prev-day = Previous day
tui-help-nav-next-day = Next day
tui-help-nav-today = Go to today
tui-help-nav-next-field = Next field
tui-help-nav-prev-field = Previous field
tui-help-nav-select = Select / confirm
tui-help-nav-cancel = Cancel / back
tui-help-section-actions = Actions
tui-help-action-start = Start timer
tui-help-action-new = New entry (with hours)
tui-help-action-edit = Edit entry
tui-help-action-delete = Delete entry
tui-help-action-pick = Open date picker
tui-help-action-refresh = Refresh
tui-help-section-general = General
tui-help-general-help = Toggle help
tui-help-general-quit = Quit

# TUI — Date Picker
tui-datepicker-sun = Su
tui-datepicker-mon = Mo
tui-datepicker-tue = Tu
tui-datepicker-wed = We
tui-datepicker-thu = Th
tui-datepicker-fri = Fr
tui-datepicker-sat = Sa



# TUI — Dashboard stats footer
tui-dash-projects = projects
tui-dash-stats-total = total
