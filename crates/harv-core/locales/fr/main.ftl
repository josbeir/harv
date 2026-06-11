# Errors
err-not-authenticated = Vous n'êtes pas authentifié. Exécutez `harv connect` pour vous connecter à votre compte Harvest.
err-config-not-found = Fichier de configuration introuvable. Exécutez `harv connect` pour vous connecter.
err-config-malformed = Le fichier de configuration est malformé : { $msg }
err-http = Erreur HTTP : { $msg }
err-api = L'API Harvest a retourné une erreur ({ $status }) : { $message }
err-io = Erreur E/S : { $msg }
err-invalid-date = Date invalide : { $msg }
err-invalid-time = Heure invalide : { $msg }
err-no-running-timer = Aucun minuteur en cours.
err-no-project-assignments = Vous n'avez aucune affectation de projet.
err-no-task-assignments = Aucune affectation de tâche trouvée pour le projet { $project_id }.
err-alias-not-found = Alias '{ $name }' introuvable. Utilisez `harv alias list` pour voir les alias.
err-oauth-failed = Le jeton d'accès n'a pas pu être récupéré depuis la réponse OAuth2.
err-oauth-denied = L'autorisation a été refusée. Réessayez avec `harv connect`.

# Date/Time
datetime-today = Aujourd'hui
datetime-at = à
datetime-am = am
datetime-pm = pm
datetime-hours-suffix = h
datetime-unknown = inconnu

# Text
text-no-client = Aucun client
text-running = En cours
text-not-running = Arrêté
text-none = —
text-unknown-hours = —
text-yes = Oui
text-no = Non

# CLI — Auth
cli-auth-opening = Ouverture du navigateur pour l'authentification Harvest…
cli-auth-manual-url = Si le navigateur ne s'ouvre pas, visitez l'URL ci-dessous.
cli-auth-success = Authentifié avec succès. Configuration enregistrée dans { $path }.
cli-auth-timeout = Connexion OAuth expirée après 120 secondes.
cli-auth-failed-bind = Échec de liaison au port { $port } : { $err }
cli-auth-failed-browser = Échec d'ouverture du navigateur : { $err }. Ouvrez cette URL manuellement :

# CLI — Connect
cli-connect-opening = Ouverture du navigateur pour l'authentification Harvest…
cli-connect-success = Authentifié avec succès. Configuration enregistrée dans { $path }.
cli-connect-failed = Échec de l'authentification : { $err }
cli-connect-save-failed = Échec de l'enregistrement de la configuration : { $err }

# CLI — Track
cli-track-loading-projects = Chargement des affectations de projet…
cli-track-project-not-found = Projet ID { $pid } introuvable dans vos affectations.
cli-track-task-not-assigned = Tâche ID { $tid } non assignée au projet { $pid }.
cli-track-creating = Création de l'entrée de temps…
cli-track-timer-started = Minuteur démarré ! { $confirmation }
cli-track-created = Créé : { $confirmation }

# CLI — Start
cli-start-delegates = Délégation à harv track…

# CLI — Stop
cli-stop-loading = Chargement…
cli-stop-no-timer = Aucun minuteur en cours.
cli-stop-prompt-which = Quel minuteur voulez-vous arrêter ?
cli-stop-prompt-notes = Notes (vide pour conserver) :
cli-stop-success = Minuteur arrêté.
cli-stop-detail =   #{ $id }	{ $client } → { $project } → { $task }	{ $hours }h

# CLI — Status
cli-status-loading = Chargement…
cli-status-no-timer = Aucun minuteur en cours.
cli-status-running-header = Minuteurs en cours :
cli-status-today-header = Entrées du jour ({ $total }h au total) :
cli-status-entry-line =   #{ $id }	{ $hours }	{ $project } → { $task }	{ $notes }

# CLI — Edit
cli-edit-loading = Chargement…
cli-edit-loading-details = Chargement des détails…
cli-edit-prompt-running = Quel minuteur voulez-vous modifier ?
cli-edit-prompt-entry = Quelle entrée voulez-vous modifier ?
cli-edit-no-entries = Aucune entrée à modifier. Utilisez `harv track` pour en créer une.
cli-edit-running-rejects-hours = Impossible de modifier les heures d'un minuteur en cours. Arrêtez-le d'abord avec `harv stop`.
cli-edit-running-rejects-date = Impossible de modifier la date d'un minuteur en cours. Arrêtez-le d'abord avec `harv stop`.
cli-edit-project-not-found = Projet ID { $pid } introuvable dans vos affectations.
cli-edit-task-not-assigned = Tâche ID { $tid } non assignée au projet { $pid }.
cli-edit-notes-prompt = Notes (actuel : "{ $existing }", vide pour conserver) :
cli-edit-saving = Enregistrement des modifications…
cli-edit-success = Mis à jour : #{ $id } — { $hours } — { $date } → { $project } → { $task }
cli-edit-entry-line = #{ $id }  { $hours }  { $project } → { $task }
cli-edit-status-running = En cours

# CLI — Note
cli-note-loading = Chargement…
cli-note-no-timer = Aucun minuteur en cours.
cli-note-prompt-which = Quel minuteur ?
cli-note-prompt-notes = Notes (vide pour conserver) :
cli-note-success = Notes mises à jour pour le minuteur #{ $id }.
cli-note-noop = Rien à mettre à jour pour le minuteur #{ $id }.

# CLI — Projects
cli-projects-loading = Chargement des affectations de projet…
cli-projects-header-client = Client
cli-projects-header-project = Projet
cli-projects-header-tasks = Tâches
cli-projects-header-id = ID Projet

# CLI — Tasks
cli-tasks-header-task = Tâche
cli-tasks-header-id = ID Tâche
cli-tasks-header-billable = Facturable

# CLI — Whoami
cli-whoami-not-auth = Non authentifié.
cli-whoami-not-auth-hint = Exécutez `harv connect` pour vous connecter avec votre compte Harvest.
cli-whoami-warning-company = Attention : impossible de récupérer les informations de l'entreprise : { $err }
cli-whoami-account-label = Authentifié sur le compte { $account_id }
cli-whoami-name = Nom :
cli-whoami-email = Email :
cli-whoami-active = Actif :
cli-whoami-timezone = Fuseau horaire :
cli-whoami-capacity = Capacité hebdomadaire :
cli-whoami-roles = Rôles d'accès :
cli-whoami-company = Entreprise :

# CLI — Disconnect
cli-disconnect-not-auth = Non authentifié. Rien à déconnecter.
cli-disconnect-disconnecting = Déconnexion du compte Harvest { $id }…
cli-disconnect-disconnecting-no-id = Déconnexion…
cli-disconnect-warning-cache = Attention : impossible de vider le cache des projets : { $err }
cli-disconnect-removed = Configuration supprimée : { $path }
cli-disconnect-cache-cleared = Cache des projets vidé.
cli-disconnect-done = Vous êtes maintenant déconnecté. Exécutez `harv connect` pour vous reconnecter.

# CLI — Config
cli-config-file = Fichier de configuration : { $path }
cli-config-not-found = (introuvable)
cli-config-access-token = access-token:
cli-config-locale = locale:
cli-config-auto-detect = (auto-detect)
cli-config-account-id = account-id:
cli-config-cache-ttl = cache-ttl:
cli-config-aliases = alias:
cli-config-none-bare = (aucun)
cli-config-locale-invalid = Invalid locale: { $value }. Supported: { $supported } (or empty/auto to reset)
cli-config-redacted = <masqué>
cli-config-unknown-setting = Paramètre inconnu : { $setting }. Paramètres valides : access-token, account-id, aliases, cache-ttl
cli-config-cache-ttl-invalid = cache-ttl doit être un nombre positif
cli-config-unknown-setting-set = Paramètre inconnu : { $setting }. Paramètres valides : cache-ttl
cli-config-set-success = { $setting } défini sur { $value }
cli-config-load-failed = Échec du chargement de la configuration : { $err }
cli-config-save-failed = Échec de l'enregistrement de la configuration : { $err }
cli-config-alias-format = { $name } -> projet: { $pid }, tâche: { $tid }

# CLI — Alias
cli-alias-loading = Chargement des affectations de projet…
cli-alias-no-assignments = Aucune affectation de projet trouvée.
cli-alias-created = Alias '{ $name }' créé : { $display } => { $task }
cli-alias-list-empty = Aucun alias défini. Utilisez `harv alias create` pour en créer un.
cli-alias-header-alias = Alias
cli-alias-header-project = Projet
cli-alias-header-task = Tâche
cli-alias-not-found = Alias '{ $name }' introuvable.
cli-alias-deleted = Alias '{ $name }' supprimé.

# CLI — Prompts
cli-prompt-alias-name = Nom de l'alias :
cli-prompt-alias-empty = Le nom de l'alias ne peut pas être vide
cli-prompt-alias-spaces = Le nom de l'alias ne peut pas contenir d'espaces
cli-prompt-project = Projet :
cli-prompt-task = Tâche :
cli-prompt-date = Date :
cli-prompt-date-future = La date ne peut pas être dans le futur
cli-prompt-date-invalid = Format de date invalide (AAAA-MM-JJ)
cli-prompt-hours = Heures (0 pour démarrer le minuteur, ex. 1,5 ou 1:30) :
cli-prompt-hours-negative = Les heures ne peuvent pas être négatives
cli-prompt-notes-editor = Notes (ouvre $EDITOR, vide pour ignorer) :
cli-prompt-notes = Notes (vide pour ignorer) :
cli-prompt-date-keep = Date (vide pour conserver) :
cli-prompt-hours-keep = Heures (vide pour conserver, 0 pour effacer) :
cli-prompt-hours-format-hint = Utilisez le format HH:MM (ex. 1:30)
cli-prompt-hours-invalid-hhmm = Heures invalides en HH:MM
cli-prompt-hours-invalid-minutes = Minutes invalides en HH:MM
cli-prompt-hours-range = Les minutes doivent être entre 0 et 59
cli-prompt-hours-non-negative = Les heures ne peuvent pas être négatives
cli-prompt-hours-format-error = Entrez un nombre valide ou le format HH:MM (ex. 1:30)
cli-prompt-date-future-error = La date { $date } est dans le futur

# TUI — App
tui-app-title = HARV
tui-app-running = ● En cours
tui-app-idle = ○ Inactif
tui-app-confirm-title = Confirmer
tui-app-confirm-prompt = y = confirmer   autre touche = annuler
tui-app-confirm-delete = "{ $desc }" Supprimer cette entrée ?
tui-app-confirm-stop-start = Un minuteur est en cours : "{ $desc }"  L'arrêter et en démarrer un nouveau ?
tui-app-loading-create = Création de l'entrée…
tui-app-loading-save = Enregistrement des modifications…
tui-app-loading-sync = Synchronisation avec Harvest…
tui-app-loading-generic = Chargement…
tui-app-loading-stop = Arrêt du minuteur…
tui-app-loading-delete = Suppression de l'entrée…

# TUI — Dashboard
tui-dash-date-prev = <
tui-dash-date-next = >
tui-dash-today = (Aujourd'hui)
tui-dash-running-header = EN COURS { $elapsed }
tui-dash-idle-header = INACTIF
tui-dash-no-timer = Aucun minuteur
tui-dash-table-project = Projet
tui-dash-table-hours = Heures
tui-dash-table-task = Tâche
tui-dash-table-notes = Notes
tui-dash-block-today = Aujourd'hui
tui-dash-hours-total = { $total }h au total
tui-dash-running-prefix = ●
tui-dash-empty-today = Aucune entrée aujourd'hui. Appuyez sur n pour démarrer !
tui-dash-empty-past = Aucune entrée pour le { $date }. Appuyez sur n pour enregistrer du temps !
tui-dash-desc-running = en cours
tui-dash-desc-stopped = arrêté

# TUI — Shortcuts
tui-short-day = Jour
tui-short-pick = Choisir
tui-short-new = Nouv
tui-short-start = Démarrer
tui-short-edit = Modif
tui-short-stop = Arrêter
tui-short-del = Suppr
tui-short-refr = Actual
tui-short-quit = Quitter
tui-short-help = Aide

# TUI — Form
tui-form-title-start = Démarrer le minuteur
tui-form-title-create = Nouvelle entrée
tui-form-title-edit = Modifier l'entrée
tui-form-date-label = Date (AAAA-MM-JJ)
tui-form-hours-label = Heures (ex. 1,5 ou 1:30)
tui-form-notes-label = Notes (optionnel)
tui-form-project-title = Projet
tui-form-project-search = Projet [{ $search }]
tui-form-task-title = Tâche
tui-form-task-search = Tâche [{ $search }]
tui-form-project-loading = Chargement des projets…
tui-form-project-empty = Aucune affectation de projet
tui-form-task-empty = Aucune tâche disponible
tui-form-task-select-first = Sélectionnez d'abord un projet
tui-form-empty-field = (vide)
tui-form-help-create = Tab: champ suivant │ Entrée: suivant/envoyer │ Échap: annuler
tui-form-help-start = Tab: champ suivant │ Entrée: démarrer │ Échap: annuler
tui-form-help-edit = Tab: champ suivant │ Entrée: enregistrer │ Échap: annuler

# TUI — Help
tui-help-title = Aide
tui-help-section-nav = Navigation
tui-help-nav-down = Descendre (listes)
tui-help-nav-up = Monter (listes)
tui-help-nav-prev-day = Jour précédent
tui-help-nav-next-day = Jour suivant
tui-help-nav-today = Aller à aujourd'hui
tui-help-nav-next-field = Champ suivant
tui-help-nav-prev-field = Champ précédent
tui-help-nav-select = Sélectionner / confirmer
tui-help-nav-cancel = Annuler / retour
tui-help-section-actions = Actions
tui-help-action-start = Démarrer le minuteur
tui-help-action-new = Nouvelle entrée (avec heures)
tui-help-action-edit = Modifier l'entrée
tui-help-action-delete = Supprimer l'entrée
tui-help-action-pick = Ouvrir le sélecteur de date
tui-help-action-refresh = Actualiser
tui-help-section-general = Général
tui-help-general-help = Afficher/masquer l'aide
tui-help-general-quit = Quitter

# TUI — Date Picker
tui-datepicker-sun = Di
tui-datepicker-mon = Lu
tui-datepicker-tue = Ma
tui-datepicker-wed = Me
tui-datepicker-thu = Je
tui-datepicker-fri = Ve
tui-datepicker-sat = Sa
