# Errors
err-not-authenticated = U bent niet geauthenticeerd. Voer `harv connect` uit om in te loggen op uw Harvest-account.
err-config-not-found = Configuratiebestand niet gevonden. Voer `harv connect` uit om in te loggen.
err-config-malformed = Configuratiebestand is ongeldig: { $msg }
err-http = HTTP-fout: { $msg }
err-api = Harvest API retourneerde een fout ({ $status }): { $message }
err-io = IO-fout: { $msg }
err-invalid-date = Ongeldige datum: { $msg }
err-invalid-time = Ongeldige tijd: { $msg }
err-no-running-timer = Er loopt geen timer.
err-no-project-assignments = U heeft geen projecttoewijzingen.
err-no-task-assignments = Geen taaktoewijzingen gevonden voor project { $project_id }.
err-alias-not-found = Alias '{ $name }' niet gevonden. Gebruik `harv alias list` om aliassen te bekijken.
err-oauth-failed = Toegangstoken kon niet worden opgehaald uit de OAuth2-respons.
err-oauth-denied = Autorisatie is geweigerd. Probeer opnieuw met `harv connect`.

# Date/Time
datetime-hours-suffix = u

# Text
text-yes = Ja
text-no = Nee

# CLI — Auth
cli-auth-manual-url = Als de browser niet opent, bezoek dan de onderstaande URL.

# CLI — Connect
cli-connect-opening = Browser openen voor Harvest-authenticatie…
cli-connect-success = Succesvol geauthenticeerd met Harvest. Configuratie opgeslagen in { $path }.
cli-connect-failed = Authenticatie mislukt: { $err }
cli-connect-save-failed = Kon configuratie niet opslaan: { $err }

# CLI — Track
cli-track-loading-projects = Projecttoewijzingen laden…
cli-track-project-not-found = Project-ID { $pid } niet gevonden in uw toewijzingen.
cli-track-task-not-assigned = Taak-ID { $tid } niet toegewezen aan project { $pid }.
cli-track-creating = Tijdregistratie aanmaken…
cli-track-timer-started = Timer gestart! { $confirmation }
cli-track-created = Aangemaakt: { $confirmation }

# CLI — Start
cli-start-delegates = Doorverwijzen naar harv track…

# CLI — Stop
cli-stop-loading = Laden…
cli-stop-no-timer = Er loopt geen timer.
cli-stop-prompt-which = Welke timer wilt u stoppen?
cli-stop-prompt-notes = Notities (leeg laten om huidige te behouden):
cli-stop-success = Timer gestopt.
cli-stop-detail =   #{ $id }	{ $client } → { $project } → { $task }	{ $hours }u

# CLI — Status
cli-status-loading = Laden…
cli-status-no-timer = Er loopt geen timer.
cli-status-running-header = Lopende timers:
cli-status-today-header = Invoer van vandaag ({ $total }u totaal):
cli-status-entry-line =   #{ $id }	{ $hours }	{ $project } → { $task }	{ $notes }

# CLI — Edit
cli-edit-loading = Laden…
cli-edit-loading-details = Detailinformatie laden…
cli-edit-prompt-running = Welke timer wilt u bewerken?
cli-edit-prompt-entry = Welke invoer wilt u bewerken?
cli-edit-no-entries = Geen invoer om te bewerken. Gebruik `harv track` om een invoer aan te maken.
cli-edit-running-rejects-hours = U kunt de uren van een lopende timer niet wijzigen. Stop deze eerst met `harv stop`.
cli-edit-running-rejects-date = U kunt de datum van een lopende timer niet wijzigen. Stop deze eerst met `harv stop`.
cli-edit-project-not-found = Project-ID { $pid } niet gevonden in uw toewijzingen.
cli-edit-task-not-assigned = Taak-ID { $tid } niet toegewezen aan project { $pid }.
cli-edit-notes-prompt = Notities (huidig: "{ $existing }", leeg laten om te behouden):
cli-edit-saving = Wijzigingen opslaan…
cli-edit-success = Bijgewerkt: #{ $id } — { $hours } — { $date } → { $project } → { $task }
cli-edit-entry-line = #{ $id }  { $hours }  { $project } → { $task }
cli-edit-status-running = Lopend

# CLI — Note
cli-note-loading = Laden…
cli-note-no-timer = Er loopt geen timer.
cli-note-prompt-which = Welke timer?
cli-note-prompt-notes = Notities (leeg laten om huidige te behouden):
cli-note-success = Notities bijgewerkt voor timer #{ $id }.
cli-note-noop = Niets om bij te werken voor timer #{ $id }.

# CLI — Projects
cli-projects-loading = Projecttoewijzingen laden…
cli-projects-header-client = Klant
cli-projects-header-project = Project
cli-projects-header-tasks = Taken
cli-projects-header-id = Project-ID

# CLI — Tasks
cli-tasks-header-task = Taak
cli-tasks-header-id = Taak-ID
cli-tasks-header-billable = Factureerbaar

# CLI — Whoami
cli-whoami-not-auth = Niet geauthenticeerd.
cli-whoami-not-auth-hint = Voer `harv connect` uit om in te loggen met uw Harvest-account.
cli-whoami-warning-company = Waarschuwing: kon bedrijfsinformatie niet ophalen: { $err }
cli-whoami-account-label = Geauthenticeerd bij account { $account_id }
cli-whoami-name = Naam:
cli-whoami-email = E-mail:
cli-whoami-active = Actief:
cli-whoami-timezone = Tijdzone:
cli-whoami-capacity = Wekelijkse capaciteit:
cli-whoami-roles = Toegangsrollen:
cli-whoami-company = Bedrijf:

# CLI — Disconnect
cli-disconnect-not-auth = Niet geauthenticeerd. Niets om te ontkoppelen.
cli-disconnect-disconnecting = Ontkoppelen van Harvest-account { $id }…
cli-disconnect-disconnecting-no-id = Ontkoppelen…
cli-disconnect-warning-cache = Waarschuwing: kon projectcache niet wissen: { $err }
cli-disconnect-removed = Configuratie verwijderd: { $path }
cli-disconnect-cache-cleared = Projectcache gewist.
cli-disconnect-done = U bent nu ontkoppeld. Voer `harv connect` uit om opnieuw in te loggen.

# CLI — Config
cli-config-file = Configuratiebestand: { $path }
cli-config-not-found = (niet gevonden)
cli-config-access-token = access-token:
cli-config-locale = locale:
cli-config-auto-detect = (auto-detect)
cli-config-account-id = account-id:
cli-config-cache-ttl = cache-ttl:
cli-config-check-updates = check-updates:
cli-config-aliases = aliassen:
cli-config-none-bare = (geen)
cli-config-locale-invalid = Invalid locale: { $value }. Supported: { $supported } (or empty/auto to reset)
cli-config-redacted = <afgeschermd>
cli-config-unknown-setting = Onbekende instelling: { $setting }. Geldige instellingen: access-token, account-id, aliases, cache-ttl, check-updates
cli-config-cache-ttl-invalid = cache-ttl moet een positief getal zijn
cli-config-check-updates-invalid = check-updates moet true of false zijn
cli-config-locale-invalid = Invalid locale: { $value }. Supported: { $supported } (or empty/auto to reset)
cli-config-unknown-setting-set = Onbekende instelling: { $setting }. Geldige instellingen: cache-ttl, locale, check-updates
cli-config-set-success = { $setting } ingesteld op { $value }
cli-config-load-failed = Kon configuratie niet laden: { $err }
cli-config-save-failed = Kon configuratie niet opslaan: { $err }
cli-config-alias-format = { $name } -> project: { $pid }, taak: { $tid }

# CLI — Alias
cli-alias-loading = Projecttoewijzingen laden…
cli-alias-no-assignments = Geen projecttoewijzingen gevonden.
cli-alias-created = Alias '{ $name }' aangemaakt: { $display } => { $task }
cli-alias-list-empty = Geen aliassen gedefinieerd. Gebruik `harv alias create` om er een aan te maken.
cli-alias-header-alias = Alias
cli-alias-header-project = Project
cli-alias-header-task = Taak
cli-alias-not-found = Alias '{ $name }' niet gevonden.
cli-alias-deleted = Alias '{ $name }' verwijderd.

# CLI — Prompts
cli-prompt-alias-name = Aliasnaam:
cli-prompt-alias-empty = Aliasnaam mag niet leeg zijn
cli-prompt-alias-spaces = Aliasnaam mag geen spaties bevatten
cli-prompt-project = Project:
cli-prompt-task = Taak:
cli-prompt-date = Datum:
cli-prompt-date-future = Datum mag niet in de toekomst liggen
cli-prompt-date-invalid = Ongeldig datumformaat (JJJJ-MM-DD)
cli-prompt-hours = Uren (0 om timer te starten, bijv. 1,5 of 1:30):
cli-prompt-hours-negative = Uren mogen niet negatief zijn
cli-prompt-notes-editor = Notities (opent $EDITOR, leeg laten om over te slaan):
cli-prompt-notes = Notities (leeg laten om over te slaan):
cli-prompt-date-keep = Datum (leeg laten om huidige te behouden):
cli-prompt-hours-keep = Uren (leeg laten om te behouden, 0 om te wissen):
cli-prompt-hours-format-hint = Gebruik UU:MM-formaat (bijv. 1:30)
cli-prompt-hours-invalid-hhmm = Ongeldige uren in UU:MM
cli-prompt-hours-invalid-minutes = Ongeldige minuten in UU:MM
cli-prompt-hours-range = Minuten moeten 0-59 zijn
cli-prompt-hours-non-negative = Uren mogen niet negatief zijn
cli-prompt-hours-format-error = Voer een geldig getal in of UU:MM-formaat (bijv. 1:30)
cli-prompt-date-future-error = Datum { $date } ligt in de toekomst

# TUI — App
tui-app-title = HARV
tui-app-running = ● Lopend
tui-app-idle = ○ Inactief
tui-app-update = v{ $version } beschikbaar
tui-app-confirm-title = Bevestigen
tui-app-confirm-prompt = y = bevestigen   andere toets = annuleren
tui-app-confirm-delete = "{ $desc }" Deze invoer verwijderen?
tui-app-confirm-stop-start = Er loopt een timer: "{ $desc }"  Stoppen en een nieuwe starten?
tui-app-loading-create = Invoer aanmaken…
tui-app-loading-save = Wijzigingen opslaan…
tui-app-loading-sync = Synchroniseren met Harvest…
tui-app-loading-generic = Laden…
tui-app-loading-stop = Timer stoppen…
tui-app-loading-delete = Invoer verwijderen…

# TUI — Dashboard
tui-dash-date-prev = <
tui-dash-date-next = >
tui-dash-today = (Vandaag)
tui-dash-running-header = LOPEND { $elapsed }
tui-dash-idle-header = INACTIEF
tui-dash-no-timer = Geen timer actief
tui-dash-table-project = Project
tui-dash-table-hours = Uren
tui-dash-table-task = Taak
tui-dash-table-notes = Notities
tui-dash-block-today = Vandaag
tui-dash-hours-total = { $total }u totaal
tui-dash-running-prefix = ●
tui-dash-empty-today = Geen invoer vandaag. Druk op n om te starten!
tui-dash-empty-past = Geen invoer voor { $date }. Druk op n om tijd te registreren!
tui-dash-desc-running = lopend
tui-dash-desc-stopped = gestopt

# TUI — Shortcuts
tui-short-day = Dag
tui-short-pick = Kiezen
tui-short-new = Nieuw
tui-short-start = Start
tui-short-edit = Bewerk
tui-short-stop = Stop
tui-short-del = Verw
tui-short-refr = Vernieuw
tui-short-quit = Sluit
tui-short-help = Help

# TUI — Form
tui-form-title-start = Timer starten
tui-form-title-create = Nieuwe invoer
tui-form-title-edit = Invoer bewerken
tui-form-date-label = Datum (JJJJ-MM-DD)
tui-form-hours-label = Uren (bijv. 1,5 of 1:30)
tui-form-notes-label = Notities (optioneel)
tui-form-project-title = Project
tui-form-project-search = Project [{ $search }]
tui-form-task-title = Taak
tui-form-task-search = Taak [{ $search }]
tui-form-project-loading = Projecten laden…
tui-form-project-empty = Geen projecttoewijzingen
tui-form-task-empty = Geen taken beschikbaar
tui-form-task-select-first = Selecteer eerst een project
tui-form-empty-field = (leeg)
tui-form-help-create = Tab: volgend veld │ Enter: volgende/verzend │ Esc: annuleren
tui-form-help-start = Tab: volgend veld │ Enter: timer starten │ Esc: annuleren
tui-form-help-edit = Tab: volgend veld │ Enter: opslaan │ Esc: annuleren

# TUI — Help
tui-help-title = Help
tui-help-section-nav = Navigatie
tui-help-nav-down = Omlaag (lijsten)
tui-help-nav-up = Omhoog (lijsten)
tui-help-nav-prev-day = Vorige dag
tui-help-nav-next-day = Volgende dag
tui-help-nav-today = Ga naar vandaag
tui-help-nav-next-field = Volgend veld
tui-help-nav-prev-field = Vorig veld
tui-help-nav-select = Selecteren / bevestigen
tui-help-nav-cancel = Annuleren / terug
tui-help-section-actions = Acties
tui-help-action-start = Timer starten
tui-help-action-new = Nieuwe invoer (met uren)
tui-help-action-edit = Invoer bewerken
tui-help-action-delete = Invoer verwijderen
tui-help-action-pick = Datumkiezer openen
tui-help-action-refresh = Vernieuwen
tui-help-section-general = Algemeen
tui-help-general-help = Help tonen/verbergen
tui-help-general-quit = Afsluiten

# TUI — Date Picker
tui-datepicker-sun = Zo
tui-datepicker-mon = Ma
tui-datepicker-tue = Di
tui-datepicker-wed = Wo
tui-datepicker-thu = Do
tui-datepicker-fri = Vr
tui-datepicker-sat = Za

# Progress steps for TUI refresh
tui-app-loading-entries = Tijdregistraties laden...
tui-app-loading-assignments = Projectgegevens laden...

# TUI — Dashboard stats footer
tui-dash-projects = projecten
tui-dash-stats-total = totaal
