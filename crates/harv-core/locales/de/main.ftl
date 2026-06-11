# Errors
err-not-authenticated = Sie sind nicht authentifiziert. Führen Sie `harv connect` aus, um sich bei Ihrem Harvest-Konto anzumelden.
err-config-not-found = Konfigurationsdatei nicht gefunden. Führen Sie `harv connect` aus, um sich anzumelden.
err-config-malformed = Konfigurationsdatei ist fehlerhaft: { $msg }
err-http = HTTP-Fehler: { $msg }
err-api = Harvest API hat einen Fehler zurückgegeben ({ $status }): { $message }
err-io = IO-Fehler: { $msg }
err-invalid-date = Ungültiges Datum: { $msg }
err-invalid-time = Ungültige Uhrzeit: { $msg }
err-no-running-timer = Kein Timer läuft.
err-no-project-assignments = Sie haben keine Projektzuweisungen.
err-no-task-assignments = Keine Aufgabenzuweisungen für Projekt { $project_id } gefunden.
err-alias-not-found = Alias '{ $name }' nicht gefunden. Verwenden Sie `harv alias list`, um Aliase anzuzeigen.
err-oauth-failed = Zugriffstoken konnte nicht aus der OAuth2-Antwort abgerufen werden.
err-oauth-denied = Autorisierung wurde verweigert. Versuchen Sie es erneut mit `harv connect`.

# Date/Time
datetime-today = Heute
datetime-at = um
datetime-hours-suffix = Std

# Text
text-no-client = Kein Kunde
text-yes = Ja
text-no = Nein

# CLI — Auth
cli-auth-manual-url = Wenn der Browser nicht öffnet, besuchen Sie die untenstehende URL.

# CLI — Connect
cli-connect-opening = Browser für Harvest-Authentifizierung wird geöffnet…
cli-connect-success = Erfolgreich bei Harvest authentifiziert. Konfiguration gespeichert in { $path }.
cli-connect-failed = Authentifizierung fehlgeschlagen: { $err }
cli-connect-save-failed = Konfiguration konnte nicht gespeichert werden: { $err }

# CLI — Track
cli-track-loading-projects = Projektzuweisungen werden geladen…
cli-track-project-not-found = Projekt-ID { $pid } nicht in Ihren Zuweisungen gefunden.
cli-track-task-not-assigned = Aufgaben-ID { $tid } nicht dem Projekt { $pid } zugewiesen.
cli-track-creating = Zeiteintrag wird erstellt…
cli-track-timer-started = Timer gestartet! { $confirmation }
cli-track-created = Erstellt: { $confirmation }

# CLI — Start
cli-start-delegates = Weiterleitung an harv track…

# CLI — Stop
cli-stop-loading = Laden…
cli-stop-no-timer = Kein Timer läuft.
cli-stop-prompt-which = Welchen Timer möchten Sie stoppen?
cli-stop-prompt-notes = Notizen (leer lassen zum Beibehalten):
cli-stop-success = Timer gestoppt.
cli-stop-detail =   #{ $id }	{ $client } → { $project } → { $task }	{ $hours }Std

# CLI — Status
cli-status-loading = Laden…
cli-status-no-timer = Kein Timer läuft.
cli-status-running-header = Laufende Timer:
cli-status-today-header = Heutige Einträge ({ $total }Std gesamt):
cli-status-entry-line =   #{ $id }	{ $hours }	{ $project } → { $task }	{ $notes }

# CLI — Edit
cli-edit-loading = Laden…
cli-edit-loading-details = Details werden geladen…
cli-edit-prompt-running = Welchen Timer möchten Sie bearbeiten?
cli-edit-prompt-entry = Welchen Eintrag möchten Sie bearbeiten?
cli-edit-no-entries = Keine Einträge zum Bearbeiten. Verwenden Sie `harv track`, um einen zu erstellen.
cli-edit-running-rejects-hours = Stunden eines laufenden Timers können nicht geändert werden. Stoppen Sie ihn zuerst mit `harv stop`.
cli-edit-running-rejects-date = Datum eines laufenden Timers kann nicht geändert werden. Stoppen Sie ihn zuerst mit `harv stop`.
cli-edit-project-not-found = Projekt-ID { $pid } nicht in Ihren Zuweisungen gefunden.
cli-edit-task-not-assigned = Aufgaben-ID { $tid } nicht dem Projekt { $pid } zugewiesen.
cli-edit-notes-prompt = Notizen (aktuell: "{ $existing }", leer lassen zum Beibehalten):
cli-edit-saving = Änderungen werden gespeichert…
cli-edit-success = Aktualisiert: #{ $id } — { $hours } — { $date } → { $project } → { $task }
cli-edit-entry-line = #{ $id }  { $hours }  { $project } → { $task }
cli-edit-status-running = Laufend

# CLI — Note
cli-note-loading = Laden…
cli-note-no-timer = Kein Timer läuft.
cli-note-prompt-which = Welcher Timer?
cli-note-prompt-notes = Notizen (leer lassen zum Beibehalten):
cli-note-success = Notizen für Timer #{ $id } aktualisiert.
cli-note-noop = Nichts zu aktualisieren für Timer #{ $id }.

# CLI — Projects
cli-projects-loading = Projektzuweisungen werden geladen…
cli-projects-header-client = Kunde
cli-projects-header-project = Projekt
cli-projects-header-tasks = Aufgaben
cli-projects-header-id = Projekt-ID

# CLI — Tasks
cli-tasks-header-task = Aufgabe
cli-tasks-header-id = Aufgaben-ID
cli-tasks-header-billable = Abrechenbar

# CLI — Whoami
cli-whoami-not-auth = Nicht authentifiziert.
cli-whoami-not-auth-hint = Führen Sie `harv connect` aus, um sich mit Ihrem Harvest-Konto anzumelden.
cli-whoami-warning-company = Warnung: Unternehmensinformationen konnten nicht abgerufen werden: { $err }
cli-whoami-account-label = Authentifiziert bei Konto { $account_id }
cli-whoami-name = Name:
cli-whoami-email = E-Mail:
cli-whoami-active = Aktiv:
cli-whoami-timezone = Zeitzone:
cli-whoami-capacity = Wöchentliche Kapazität:
cli-whoami-roles = Zugriffsrollen:
cli-whoami-company = Unternehmen:

# CLI — Disconnect
cli-disconnect-not-auth = Nicht authentifiziert. Nichts zu trennen.
cli-disconnect-disconnecting = Trennung von Harvest-Konto { $id }…
cli-disconnect-disconnecting-no-id = Trennung…
cli-disconnect-warning-cache = Warnung: Projektcache konnte nicht geleert werden: { $err }
cli-disconnect-removed = Konfiguration entfernt: { $path }
cli-disconnect-cache-cleared = Projektcache geleert.
cli-disconnect-done = Sie sind jetzt getrennt. Führen Sie `harv connect` aus, um sich wieder anzumelden.

# CLI — Config
cli-config-file = Konfigurationsdatei: { $path }
cli-config-not-found = (nicht gefunden)
cli-config-access-token = access-token:
cli-config-locale = locale:
cli-config-auto-detect = (auto-detect)
cli-config-account-id = account-id:
cli-config-cache-ttl = cache-ttl:
cli-config-aliases = aliase:
cli-config-none-bare = (keine)
cli-config-locale-invalid = Invalid locale: { $value }. Supported: { $supported } (or empty/auto to reset)
cli-config-redacted = <geschwärzt>
cli-config-unknown-setting = Unbekannte Einstellung: { $setting }. Gültige Einstellungen: access-token, account-id, aliases, cache-ttl
cli-config-cache-ttl-invalid = cache-ttl muss eine positive Zahl sein
cli-config-unknown-setting-set = Unbekannte Einstellung: { $setting }. Gültige Einstellungen: cache-ttl
cli-config-set-success = { $setting } auf { $value } gesetzt
cli-config-load-failed = Konfiguration konnte nicht geladen werden: { $err }
cli-config-save-failed = Konfiguration konnte nicht gespeichert werden: { $err }
cli-config-alias-format = { $name } -> projekt: { $pid }, aufgabe: { $tid }

# CLI — Alias
cli-alias-loading = Projektzuweisungen werden geladen…
cli-alias-no-assignments = Keine Projektzuweisungen gefunden.
cli-alias-created = Alias '{ $name }' erstellt: { $display } => { $task }
cli-alias-list-empty = Keine Aliase definiert. Verwenden Sie `harv alias create`, um einen zu erstellen.
cli-alias-header-alias = Alias
cli-alias-header-project = Projekt
cli-alias-header-task = Aufgabe
cli-alias-not-found = Alias '{ $name }' nicht gefunden.
cli-alias-deleted = Alias '{ $name }' gelöscht.

# CLI — Prompts
cli-prompt-alias-name = Alias-Name:
cli-prompt-alias-empty = Alias-Name darf nicht leer sein
cli-prompt-alias-spaces = Alias-Name darf keine Leerzeichen enthalten
cli-prompt-project = Projekt:
cli-prompt-task = Aufgabe:
cli-prompt-date = Datum:
cli-prompt-date-future = Datum darf nicht in der Zukunft liegen
cli-prompt-date-invalid = Ungültiges Datumsformat (JJJJ-MM-TT)
cli-prompt-hours = Stunden (0 zum Starten des Timers, z.B. 1,5 oder 1:30):
cli-prompt-hours-negative = Stunden dürfen nicht negativ sein
cli-prompt-notes-editor = Notizen (öffnet $EDITOR, leer zum Überspringen):
cli-prompt-notes = Notizen (leer zum Überspringen):
cli-prompt-date-keep = Datum (leer lassen zum Beibehalten):
cli-prompt-hours-keep = Stunden (leer lassen zum Beibehalten, 0 zum Löschen):
cli-prompt-hours-format-hint = Verwenden Sie SS:MM-Format (z.B. 1:30)
cli-prompt-hours-invalid-hhmm = Ungültige Stunden in SS:MM
cli-prompt-hours-invalid-minutes = Ungültige Minuten in SS:MM
cli-prompt-hours-range = Minuten müssen 0-59 sein
cli-prompt-hours-non-negative = Stunden dürfen nicht negativ sein
cli-prompt-hours-format-error = Geben Sie eine gültige Zahl oder SS:MM-Format ein (z.B. 1:30)
cli-prompt-date-future-error = Datum { $date } liegt in der Zukunft

# TUI — App
tui-app-title = HARV
tui-app-running = ● Laufend
tui-app-idle = ○ Inaktiv
tui-app-confirm-title = Bestätigen
tui-app-confirm-prompt = y = bestätigen   andere Taste = abbrechen
tui-app-confirm-delete = "{ $desc }" Diesen Eintrag löschen?
tui-app-confirm-stop-start = Ein Timer läuft: "{ $desc }"  Stoppen und neuen starten?
tui-app-loading-create = Eintrag wird erstellt…
tui-app-loading-save = Änderungen werden gespeichert…
tui-app-loading-sync = Synchronisiere mit Harvest…
tui-app-loading-generic = Laden…
tui-app-loading-stop = Timer wird gestoppt…
tui-app-loading-delete = Eintrag wird gelöscht…

# TUI — Dashboard
tui-dash-date-prev = <
tui-dash-date-next = >
tui-dash-today = (Heute)
tui-dash-running-header = LÄUFT { $elapsed }
tui-dash-idle-header = INAKTIV
tui-dash-no-timer = Kein Timer aktiv
tui-dash-table-project = Projekt
tui-dash-table-hours = Stunden
tui-dash-table-task = Aufgabe
tui-dash-table-notes = Notizen
tui-dash-block-today = Heute
tui-dash-hours-total = { $total }Std gesamt
tui-dash-running-prefix = ●
tui-dash-empty-today = Keine Einträge heute. Drücken Sie n zum Starten!
tui-dash-empty-past = Keine Einträge für { $date }. Drücken Sie n zum Erfassen!
tui-dash-desc-running = laufend
tui-dash-desc-stopped = gestoppt

# TUI — Shortcuts
tui-short-day = Tag
tui-short-pick = Wählen
tui-short-new = Neu
tui-short-start = Start
tui-short-edit = Bearb
tui-short-stop = Stopp
tui-short-del = Lösch
tui-short-refr = Aktual
tui-short-quit = Ende
tui-short-help = Hilfe

# TUI — Form
tui-form-title-start = Timer starten
tui-form-title-create = Neuer Eintrag
tui-form-title-edit = Eintrag bearbeiten
tui-form-date-label = Datum (JJJJ-MM-TT)
tui-form-hours-label = Stunden (z.B. 1,5 oder 1:30)
tui-form-notes-label = Notizen (optional)
tui-form-project-title = Projekt
tui-form-project-search = Projekt [{ $search }]
tui-form-task-title = Aufgabe
tui-form-task-search = Aufgabe [{ $search }]
tui-form-project-loading = Projekte werden geladen…
tui-form-project-empty = Keine Projektzuweisungen
tui-form-task-empty = Keine Aufgaben verfügbar
tui-form-task-select-first = Zuerst ein Projekt auswählen
tui-form-empty-field = (leer)
tui-form-help-create = Tab: nächstes Feld │ Enter: weiter/senden │ Esc: abbrechen
tui-form-help-start = Tab: nächstes Feld │ Enter: Timer starten │ Esc: abbrechen
tui-form-help-edit = Tab: nächstes Feld │ Enter: speichern │ Esc: abbrechen

# TUI — Help
tui-help-title = Hilfe
tui-help-section-nav = Navigation
tui-help-nav-down = Runter (Listen)
tui-help-nav-up = Hoch (Listen)
tui-help-nav-prev-day = Vorheriger Tag
tui-help-nav-next-day = Nächster Tag
tui-help-nav-today = Zum heutigen Tag
tui-help-nav-next-field = Nächstes Feld
tui-help-nav-prev-field = Vorheriges Feld
tui-help-nav-select = Auswählen / bestätigen
tui-help-nav-cancel = Abbrechen / zurück
tui-help-section-actions = Aktionen
tui-help-action-start = Timer starten
tui-help-action-new = Neuer Eintrag (mit Stunden)
tui-help-action-edit = Eintrag bearbeiten
tui-help-action-delete = Eintrag löschen
tui-help-action-pick = Datumsauswahl öffnen
tui-help-action-refresh = Aktualisieren
tui-help-section-general = Allgemein
tui-help-general-help = Hilfe umschalten
tui-help-general-quit = Beenden

# TUI — Date Picker
tui-datepicker-sun = So
tui-datepicker-mon = Mo
tui-datepicker-tue = Di
tui-datepicker-wed = Mi
tui-datepicker-thu = Do
tui-datepicker-fri = Fr
tui-datepicker-sat = Sa
