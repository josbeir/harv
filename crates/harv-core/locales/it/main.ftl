# Errors
err-not-authenticated = Non sei autenticato. Esegui `harv connect` per accedere al tuo account Harvest.
err-config-not-found = File di configurazione non trovato. Esegui `harv connect` per accedere.
err-config-malformed = Il file di configurazione non è valido: { $msg }
err-http = Errore HTTP: { $msg }
err-api = L'API Harvest ha restituito un errore ({ $status }): { $message }
err-io = Errore I/O: { $msg }
err-invalid-date = Data non valida: { $msg }
err-invalid-time = Ora non valida: { $msg }
err-no-running-timer = Nessun timer in esecuzione.
err-no-project-assignments = Non hai assegnazioni di progetto.
err-no-task-assignments = Nessuna assegnazione di attività trovata per il progetto { $project_id }.
err-alias-not-found = Alias '{ $name }' non trovato. Usa `harv alias list` per visualizzare gli alias.
err-oauth-failed = Impossibile recuperare il token di accesso dalla risposta OAuth2.
err-oauth-denied = L'autorizzazione è stata negata. Riprova con `harv connect`.

# Date/Time
datetime-today = Oggi
datetime-at = alle
datetime-hours-suffix = h

# Text
text-no-client = Nessun cliente
text-yes = Sì
text-no = No

# CLI — Auth
cli-auth-manual-url = Se il browser non si apre, visita l'URL mostrato sotto.

# CLI — Connect
cli-connect-opening = Apertura browser per l'autenticazione Harvest…
cli-connect-success = Autenticato con successo su Harvest. Configurazione salvata in { $path }.
cli-connect-failed = Autenticazione fallita: { $err }
cli-connect-save-failed = Impossibile salvare la configurazione: { $err }

# CLI — Track
cli-track-loading-projects = Caricamento assegnazioni di progetto…
cli-track-project-not-found = Progetto ID { $pid } non trovato nelle tue assegnazioni.
cli-track-task-not-assigned = Attività ID { $tid } non assegnata al progetto { $pid }.
cli-track-creating = Creazione registrazione tempo…
cli-track-timer-started = Timer avviato! { $confirmation }
cli-track-created = Creato: { $confirmation }

# CLI — Start
cli-start-delegates = Delega a harv track…

# CLI — Stop
cli-stop-loading = Caricamento…
cli-stop-no-timer = Nessun timer in esecuzione.
cli-stop-prompt-which = Quale timer vuoi fermare?
cli-stop-prompt-notes = Note (vuoto per mantenere):
cli-stop-success = Timer fermato.
cli-stop-detail =   #{ $id }	{ $client } → { $project } → { $task }	{ $hours }h

# CLI — Status
cli-status-loading = Caricamento…
cli-status-no-timer = Nessun timer in esecuzione.
cli-status-running-header = Timer in esecuzione:
cli-status-today-header = Registrazioni di oggi ({ $total }h totali):
cli-status-entry-line =   #{ $id }	{ $hours }	{ $project } → { $task }	{ $notes }

# CLI — Edit
cli-edit-loading = Caricamento…
cli-edit-loading-details = Caricamento dettagli registrazione…
cli-edit-prompt-running = Quale timer vuoi modificare?
cli-edit-prompt-entry = Quale registrazione vuoi modificare?
cli-edit-no-entries = Nessuna registrazione da modificare. Usa `harv track` per crearne una.
cli-edit-running-rejects-hours = Impossibile modificare le ore di un timer in esecuzione. Fermalo prima con `harv stop`.
cli-edit-running-rejects-date = Impossibile modificare la data di un timer in esecuzione. Fermalo prima con `harv stop`.
cli-edit-project-not-found = Progetto ID { $pid } non trovato nelle tue assegnazioni.
cli-edit-task-not-assigned = Attività ID { $tid } non assegnata al progetto { $pid }.
cli-edit-notes-prompt = Note (attuale: "{ $existing }", vuoto per mantenere):
cli-edit-saving = Salvataggio modifiche…
cli-edit-success = Aggiornato: #{ $id } — { $hours } — { $date } → { $project } → { $task }
cli-edit-entry-line = #{ $id }  { $hours }  { $project } → { $task }
cli-edit-status-running = In esecuzione

# CLI — Note
cli-note-loading = Caricamento…
cli-note-no-timer = Nessun timer in esecuzione.
cli-note-prompt-which = Quale timer?
cli-note-prompt-notes = Note (vuoto per mantenere):
cli-note-success = Note aggiornate per il timer #{ $id }.
cli-note-noop = Niente da aggiornare per il timer #{ $id }.

# CLI — Projects
cli-projects-loading = Caricamento assegnazioni di progetto…
cli-projects-header-client = Cliente
cli-projects-header-project = Progetto
cli-projects-header-tasks = Attività
cli-projects-header-id = ID Progetto

# CLI — Tasks
cli-tasks-header-task = Attività
cli-tasks-header-id = ID Attività
cli-tasks-header-billable = Fatturabile

# CLI — Whoami
cli-whoami-not-auth = Non autenticato.
cli-whoami-not-auth-hint = Esegui `harv connect` per accedere con il tuo account Harvest.
cli-whoami-warning-company = Attenzione: impossibile recuperare le informazioni aziendali: { $err }
cli-whoami-account-label = Autenticato sull'account { $account_id }
cli-whoami-name = Nome:
cli-whoami-email = Email:
cli-whoami-active = Attivo:
cli-whoami-timezone = Fuso orario:
cli-whoami-capacity = Capacità settimanale:
cli-whoami-roles = Ruoli di accesso:
cli-whoami-company = Azienda:

# CLI — Disconnect
cli-disconnect-not-auth = Non autenticato. Niente da disconnettere.
cli-disconnect-disconnecting = Disconnessione dall'account Harvest { $id }…
cli-disconnect-disconnecting-no-id = Disconnessione…
cli-disconnect-warning-cache = Attenzione: impossibile svuotare la cache dei progetti: { $err }
cli-disconnect-removed = Configurazione rimossa: { $path }
cli-disconnect-cache-cleared = Cache dei progetti svuotata.
cli-disconnect-done = Ora sei disconnesso. Esegui `harv connect` per riaccedere.

# CLI — Config
cli-config-file = File di configurazione: { $path }
cli-config-not-found = (non trovato)
cli-config-access-token = access-token:
cli-config-locale = locale:
cli-config-auto-detect = (auto-detect)
cli-config-account-id = account-id:
cli-config-cache-ttl = cache-ttl:
cli-config-aliases = alias:
cli-config-none-bare = (nessuno)
cli-config-locale-invalid = Invalid locale: { $value }. Supported: { $supported } (or empty/auto to reset)
cli-config-redacted = <oscurato>
cli-config-unknown-setting = Impostazione sconosciuta: { $setting }. Impostazioni valide: access-token, account-id, aliases, cache-ttl
cli-config-cache-ttl-invalid = cache-ttl deve essere un numero positivo
cli-config-unknown-setting-set = Impostazione sconosciuta: { $setting }. Impostazioni valide: cache-ttl
cli-config-set-success = { $setting } impostato su { $value }
cli-config-load-failed = Impossibile caricare la configurazione: { $err }
cli-config-save-failed = Impossibile salvare la configurazione: { $err }
cli-config-alias-format = { $name } -> progetto: { $pid }, attività: { $tid }

# CLI — Alias
cli-alias-loading = Caricamento assegnazioni di progetto…
cli-alias-no-assignments = Nessuna assegnazione di progetto trovata.
cli-alias-created = Alias '{ $name }' creato: { $display } => { $task }
cli-alias-list-empty = Nessun alias definito. Usa `harv alias create` per crearne uno.
cli-alias-header-alias = Alias
cli-alias-header-project = Progetto
cli-alias-header-task = Attività
cli-alias-not-found = Alias '{ $name }' non trovato.
cli-alias-deleted = Alias '{ $name }' eliminato.

# CLI — Prompts
cli-prompt-alias-name = Nome alias:
cli-prompt-alias-empty = Il nome dell'alias non può essere vuoto
cli-prompt-alias-spaces = Il nome dell'alias non può contenere spazi
cli-prompt-project = Progetto:
cli-prompt-task = Attività:
cli-prompt-date = Data:
cli-prompt-date-future = La data non può essere nel futuro
cli-prompt-date-invalid = Formato data non valido (AAAA-MM-GG)
cli-prompt-hours = Ore (0 per avviare il timer, es. 1.5 o 1:30):
cli-prompt-hours-negative = Le ore non possono essere negative
cli-prompt-notes-editor = Note (apre $EDITOR, vuoto per saltare):
cli-prompt-notes = Note (vuoto per saltare):
cli-prompt-date-keep = Data (vuoto per mantenere):
cli-prompt-hours-keep = Ore (vuoto per mantenere, 0 per azzerare):
cli-prompt-hours-format-hint = Usa formato HH:MM (es. 1:30)
cli-prompt-hours-invalid-hhmm = Ore non valide in HH:MM
cli-prompt-hours-invalid-minutes = Minuti non validi in HH:MM
cli-prompt-hours-range = I minuti devono essere 0-59
cli-prompt-hours-non-negative = Le ore non possono essere negative
cli-prompt-hours-format-error = Inserisci un numero valido o formato HH:MM (es. 1:30)
cli-prompt-date-future-error = La data { $date } è nel futuro

# TUI — App
tui-app-title = HARV
tui-app-running = ● In esecuzione
tui-app-idle = ○ Inattivo
tui-app-confirm-title = Conferma
tui-app-confirm-prompt = y = conferma   altro tasto = annulla
tui-app-confirm-delete = "{ $desc }" Eliminare questa registrazione?
tui-app-confirm-stop-start = C'è un timer in esecuzione: "{ $desc }"  Fermarlo e avviarne uno nuovo?
tui-app-loading-create = Creazione registrazione…
tui-app-loading-save = Salvataggio modifiche…
tui-app-loading-sync = Sincronizzazione con Harvest…
tui-app-loading-generic = Caricamento…
tui-app-loading-stop = Arresto timer…
tui-app-loading-delete = Eliminazione registrazione…

# TUI — Dashboard
tui-dash-date-prev = <
tui-dash-date-next = >
tui-dash-today = (Oggi)
tui-dash-running-header = IN ESECUZIONE { $elapsed }
tui-dash-idle-header = INATTIVO
tui-dash-no-timer = Nessun timer attivo
tui-dash-table-project = Progetto
tui-dash-table-hours = Ore
tui-dash-table-task = Attività
tui-dash-table-notes = Note
tui-dash-block-today = Oggi
tui-dash-hours-total = { $total }h totali
tui-dash-running-prefix = ●
tui-dash-empty-today = Nessuna registrazione oggi. Premi n per iniziare!
tui-dash-empty-past = Nessuna registrazione per { $date }. Premi n per registrare il tempo!
tui-dash-desc-running = in esecuzione
tui-dash-desc-stopped = fermo

# TUI — Shortcuts
tui-short-day = Giorno
tui-short-pick = Scegli
tui-short-new = Nuovo
tui-short-start = Avvia
tui-short-edit = Modifica
tui-short-stop = Ferma
tui-short-del = Elim
tui-short-refr = Aggio
tui-short-quit = Esci
tui-short-help = Aiuto

# TUI — Form
tui-form-title-start = Avvia timer
tui-form-title-create = Nuova registrazione
tui-form-title-edit = Modifica registrazione
tui-form-date-label = Data (AAAA-MM-GG)
tui-form-hours-label = Ore (es. 1.5 o 1:30)
tui-form-notes-label = Note (opzionale)
tui-form-project-title = Progetto
tui-form-project-search = Progetto [{ $search }]
tui-form-task-title = Attività
tui-form-task-search = Attività [{ $search }]
tui-form-project-loading = Caricamento progetti…
tui-form-project-empty = Nessuna assegnazione di progetto
tui-form-task-empty = Nessuna attività disponibile
tui-form-task-select-first = Seleziona prima un progetto
tui-form-empty-field = (vuoto)
tui-form-help-create = Tab: campo successivo │ Invio: avanti/invia │ Esc: annulla
tui-form-help-start = Tab: campo successivo │ Invio: avvia timer │ Esc: annulla
tui-form-help-edit = Tab: campo successivo │ Invio: salva │ Esc: annulla

# TUI — Help
tui-help-title = Aiuto
tui-help-section-nav = Navigazione
tui-help-nav-down = Giù (liste)
tui-help-nav-up = Su (liste)
tui-help-nav-prev-day = Giorno precedente
tui-help-nav-next-day = Giorno successivo
tui-help-nav-today = Vai a oggi
tui-help-nav-next-field = Campo successivo
tui-help-nav-prev-field = Campo precedente
tui-help-nav-select = Seleziona / conferma
tui-help-nav-cancel = Annulla / indietro
tui-help-section-actions = Azioni
tui-help-action-start = Avvia timer
tui-help-action-new = Nuova registrazione (con ore)
tui-help-action-edit = Modifica registrazione
tui-help-action-delete = Elimina registrazione
tui-help-action-pick = Apri selettore data
tui-help-action-refresh = Aggiorna
tui-help-section-general = Generale
tui-help-general-help = Mostra/nascondi aiuto
tui-help-general-quit = Esci

# TUI — Date Picker
tui-datepicker-sun = Do
tui-datepicker-mon = Lu
tui-datepicker-tue = Ma
tui-datepicker-wed = Me
tui-datepicker-thu = Gi
tui-datepicker-fri = Ve
tui-datepicker-sat = Sa
