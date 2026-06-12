# Errors
err-not-authenticated = No está autenticado. Ejecute `harv connect` para iniciar sesión en su cuenta de Harvest.
err-config-not-found = Archivo de configuración no encontrado. Ejecute `harv connect` para iniciar sesión.
err-config-malformed = El archivo de configuración es inválido: { $msg }
err-http = Error HTTP: { $msg }
err-api = La API de Harvest devolvió un error ({ $status }): { $message }
err-io = Error E/S: { $msg }
err-invalid-date = Fecha inválida: { $msg }
err-invalid-time = Hora inválida: { $msg }
err-no-running-timer = No hay ningún temporizador en marcha.
err-no-project-assignments = No tiene asignaciones de proyecto.
err-no-task-assignments = No se encontraron tareas asignadas para el proyecto { $project_id }.
err-alias-not-found = Alias '{ $name }' no encontrado. Use `harv alias list` para ver los alias.
err-oauth-failed = No se pudo obtener el token de acceso de la respuesta OAuth2.
err-oauth-denied = La autorización fue denegada. Inténtelo de nuevo con `harv connect`.

# Date/Time
datetime-hours-suffix = h

# Text
text-yes = Sí
text-no = No

# CLI — Auth
cli-auth-manual-url = Si el navegador no se abre, visite la URL indicada abajo.

# CLI — Connect
cli-connect-opening = Abriendo navegador para autenticación de Harvest…
cli-connect-success = Autenticado con éxito en Harvest. Configuración guardada en { $path }.
cli-connect-failed = Falló la autenticación: { $err }
cli-connect-save-failed = No se pudo guardar la configuración: { $err }

# CLI — Track
cli-track-loading-projects = Cargando asignaciones de proyecto…
cli-track-project-not-found = Proyecto ID { $pid } no encontrado en sus asignaciones.
cli-track-task-not-assigned = Tarea ID { $tid } no asignada al proyecto { $pid }.
cli-track-creating = Creando entrada de tiempo…
cli-track-timer-started = ¡Temporizador iniciado! { $confirmation }
cli-track-created = Creado: { $confirmation }

# CLI — Start
cli-start-delegates = Delegando a harv track…

# CLI — Stop
cli-stop-loading = Cargando…
cli-stop-no-timer = No hay ningún temporizador en marcha.
cli-stop-prompt-which = ¿Qué temporizador desea detener?
cli-stop-prompt-notes = Notas (vacío para mantener):
cli-stop-success = Temporizador detenido.
cli-stop-detail =   #{ $id }	{ $client } → { $project } → { $task }	{ $hours }h

# CLI — Status
cli-status-loading = Cargando…
cli-status-no-timer = No hay ningún temporizador en marcha.
cli-status-running-header = Temporizadores en marcha:
cli-status-today-header = Entradas de hoy ({ $total }h total):
cli-status-entry-line =   #{ $id }	{ $hours }	{ $project } → { $task }	{ $notes }

# CLI — Edit
cli-edit-loading = Cargando…
cli-edit-loading-details = Cargando detalles de la entrada…
cli-edit-prompt-running = ¿Qué temporizador desea editar?
cli-edit-prompt-entry = ¿Qué entrada desea editar?
cli-edit-no-entries = No hay entradas para editar. Use `harv track` para crear una.
cli-edit-running-rejects-hours = No se pueden cambiar las horas de un temporizador en marcha. Deténgalo primero con `harv stop`.
cli-edit-running-rejects-date = No se puede cambiar la fecha de un temporizador en marcha. Deténgalo primero con `harv stop`.
cli-edit-project-not-found = Proyecto ID { $pid } no encontrado en sus asignaciones.
cli-edit-task-not-assigned = Tarea ID { $tid } no asignada al proyecto { $pid }.
cli-edit-notes-prompt = Notas (actual: "{ $existing }", vacío para mantener):
cli-edit-saving = Guardando cambios…
cli-edit-success = Actualizado: #{ $id } — { $hours } — { $date } → { $project } → { $task }
cli-edit-entry-line = #{ $id }  { $hours }  { $project } → { $task }
cli-edit-status-running = En marcha

# CLI — Note
cli-note-loading = Cargando…
cli-note-no-timer = No hay ningún temporizador en marcha.
cli-note-prompt-which = ¿Qué temporizador?
cli-note-prompt-notes = Notas (vacío para mantener):
cli-note-success = Notas actualizadas para el temporizador #{ $id }.
cli-note-noop = Nada que actualizar para el temporizador #{ $id }.

# CLI — Projects
cli-projects-loading = Cargando asignaciones de proyecto…
cli-projects-header-client = Cliente
cli-projects-header-project = Proyecto
cli-projects-header-tasks = Tareas
cli-projects-header-id = ID Proyecto

# CLI — Tasks
cli-tasks-header-task = Tarea
cli-tasks-header-id = ID Tarea
cli-tasks-header-billable = Facturable

# CLI — Whoami
cli-whoami-not-auth = No autenticado.
cli-whoami-not-auth-hint = Ejecute `harv connect` para iniciar sesión con su cuenta de Harvest.
cli-whoami-warning-company = Advertencia: no se pudo obtener información de la empresa: { $err }
cli-whoami-account-label = Autenticado en la cuenta { $account_id }
cli-whoami-name = Nombre:
cli-whoami-email = Correo:
cli-whoami-active = Activo:
cli-whoami-timezone = Zona horaria:
cli-whoami-capacity = Capacidad semanal:
cli-whoami-roles = Roles de acceso:
cli-whoami-company = Empresa:

# CLI — Disconnect
cli-disconnect-not-auth = No autenticado. Nada que desconectar.
cli-disconnect-disconnecting = Desconectando de la cuenta Harvest { $id }…
cli-disconnect-disconnecting-no-id = Desconectando…
cli-disconnect-warning-cache = Advertencia: no se pudo limpiar la caché de proyectos: { $err }
cli-disconnect-removed = Configuración eliminada: { $path }
cli-disconnect-cache-cleared = Caché de proyectos limpiada.
cli-disconnect-done = Ahora está desconectado. Ejecute `harv connect` para volver a iniciar sesión.

# CLI — Config
cli-config-file = Archivo de configuración: { $path }
cli-config-not-found = (no encontrado)
cli-config-access-token = access-token:
cli-config-locale = locale:
cli-config-auto-detect = (auto-detect)
cli-config-account-id = account-id:
cli-config-cache-ttl = cache-ttl:
cli-config-aliases = alias:
cli-config-none-bare = (ninguno)
cli-config-locale-invalid = Invalid locale: { $value }. Supported: { $supported } (or empty/auto to reset)
cli-config-redacted = <censurado>
cli-config-unknown-setting = Configuración desconocida: { $setting }. Configuraciones válidas: access-token, account-id, aliases, cache-ttl
cli-config-cache-ttl-invalid = cache-ttl debe ser un número positivo
cli-config-unknown-setting-set = Configuración desconocida: { $setting }. Configuraciones válidas: cache-ttl
cli-config-set-success = { $setting } establecido a { $value }
cli-config-load-failed = No se pudo cargar la configuración: { $err }
cli-config-save-failed = No se pudo guardar la configuración: { $err }
cli-config-alias-format = { $name } -> proyecto: { $pid }, tarea: { $tid }

# CLI — Alias
cli-alias-loading = Cargando asignaciones de proyecto…
cli-alias-no-assignments = No se encontraron asignaciones de proyecto.
cli-alias-created = Alias '{ $name }' creado: { $display } => { $task }
cli-alias-list-empty = No hay alias definidos. Use `harv alias create` para crear uno.
cli-alias-header-alias = Alias
cli-alias-header-project = Proyecto
cli-alias-header-task = Tarea
cli-alias-not-found = Alias '{ $name }' no encontrado.
cli-alias-deleted = Alias '{ $name }' eliminado.

# CLI — Prompts
cli-prompt-alias-name = Nombre del alias:
cli-prompt-alias-empty = El nombre del alias no puede estar vacío
cli-prompt-alias-spaces = El nombre del alias no puede contener espacios
cli-prompt-project = Proyecto:
cli-prompt-task = Tarea:
cli-prompt-date = Fecha:
cli-prompt-date-future = La fecha no puede estar en el futuro
cli-prompt-date-invalid = Formato de fecha inválido (AAAA-MM-DD)
cli-prompt-hours = Horas (0 para iniciar temporizador, ej. 1.5 o 1:30):
cli-prompt-hours-negative = Las horas no pueden ser negativas
cli-prompt-notes-editor = Notas (abre $EDITOR, vacío para omitir):
cli-prompt-notes = Notas (vacío para omitir):
cli-prompt-date-keep = Fecha (vacío para mantener actual):
cli-prompt-hours-keep = Horas (vacío para mantener, 0 para borrar):
cli-prompt-hours-format-hint = Use formato HH:MM (ej. 1:30)
cli-prompt-hours-invalid-hhmm = Horas inválidas en HH:MM
cli-prompt-hours-invalid-minutes = Minutos inválidos en HH:MM
cli-prompt-hours-range = Los minutos deben ser 0-59
cli-prompt-hours-non-negative = Las horas no pueden ser negativas
cli-prompt-hours-format-error = Ingrese un número válido o formato HH:MM (ej. 1:30)
cli-prompt-date-future-error = La fecha { $date } está en el futuro

# TUI — App
tui-app-title = HARV
tui-app-running = ● En marcha
tui-app-idle = ○ Inactivo
tui-app-confirm-title = Confirmar
tui-app-confirm-prompt = y = confirmar   otra tecla = cancelar
tui-app-confirm-delete = "{ $desc }" ¿Eliminar esta entrada?
tui-app-confirm-stop-start = Hay un temporizador en marcha: "{ $desc }"  ¿Detenerlo e iniciar uno nuevo?
tui-app-loading-create = Creando entrada…
tui-app-loading-save = Guardando cambios…
tui-app-loading-sync = Sincronizando con Harvest…
tui-app-loading-generic = Cargando…
tui-app-loading-stop = Deteniendo temporizador…
tui-app-loading-delete = Eliminando entrada…

# TUI — Dashboard
tui-dash-date-prev = <
tui-dash-date-next = >
tui-dash-today = (Hoy)
tui-dash-running-header = EN MARCHA { $elapsed }
tui-dash-idle-header = INACTIVO
tui-dash-no-timer = Sin temporizador activo
tui-dash-table-project = Proyecto
tui-dash-table-hours = Horas
tui-dash-table-task = Tarea
tui-dash-table-notes = Notas
tui-dash-block-today = Hoy
tui-dash-hours-total = { $total }h total
tui-dash-running-prefix = ●
tui-dash-empty-today = ¡No hay entradas hoy. Presione n para comenzar!
tui-dash-empty-past = No hay entradas para { $date }. ¡Presione n para registrar tiempo!
tui-dash-desc-running = en marcha
tui-dash-desc-stopped = detenido

# TUI — Shortcuts
tui-short-day = Día
tui-short-pick = Elegir
tui-short-new = Nuevo
tui-short-start = Iniciar
tui-short-edit = Editar
tui-short-stop = Detener
tui-short-del = Elim
tui-short-refr = Actual
tui-short-quit = Salir
tui-short-help = Ayuda

# TUI — Form
tui-form-title-start = Iniciar temporizador
tui-form-title-create = Nueva entrada
tui-form-title-edit = Editar entrada
tui-form-date-label = Fecha (AAAA-MM-DD)
tui-form-hours-label = Horas (ej. 1.5 o 1:30)
tui-form-notes-label = Notas (opcional)
tui-form-project-title = Proyecto
tui-form-project-search = Proyecto [{ $search }]
tui-form-task-title = Tarea
tui-form-task-search = Tarea [{ $search }]
tui-form-project-loading = Cargando proyectos…
tui-form-project-empty = Sin asignaciones de proyecto
tui-form-task-empty = No hay tareas disponibles
tui-form-task-select-first = Seleccione un proyecto primero
tui-form-empty-field = (vacío)
tui-form-help-create = Tab: siguiente campo │ Enter: siguiente/enviar │ Esc: cancelar
tui-form-help-start = Tab: siguiente campo │ Enter: iniciar temporizador │ Esc: cancelar
tui-form-help-edit = Tab: siguiente campo │ Enter: guardar │ Esc: cancelar

# TUI — Help
tui-help-title = Ayuda
tui-help-section-nav = Navegación
tui-help-nav-down = Bajar (listas)
tui-help-nav-up = Subir (listas)
tui-help-nav-prev-day = Día anterior
tui-help-nav-next-day = Día siguiente
tui-help-nav-today = Ir a hoy
tui-help-nav-next-field = Siguiente campo
tui-help-nav-prev-field = Campo anterior
tui-help-nav-select = Seleccionar / confirmar
tui-help-nav-cancel = Cancelar / volver
tui-help-section-actions = Acciones
tui-help-action-start = Iniciar temporizador
tui-help-action-new = Nueva entrada (con horas)
tui-help-action-edit = Editar entrada
tui-help-action-delete = Eliminar entrada
tui-help-action-pick = Abrir selector de fecha
tui-help-action-refresh = Actualizar
tui-help-section-general = General
tui-help-general-help = Mostrar/ocultar ayuda
tui-help-general-quit = Salir

# TUI — Date Picker
tui-datepicker-sun = Do
tui-datepicker-mon = Lu
tui-datepicker-tue = Ma
tui-datepicker-wed = Mi
tui-datepicker-thu = Ju
tui-datepicker-fri = Vi
tui-datepicker-sat = Sá

# Progress steps for TUI refresh
tui-app-loading-entries = Cargando registros de tiempo...
tui-app-loading-assignments = Cargando datos del proyecto...

# TUI — Dashboard stats footer
tui-dash-projects = proyectos
tui-dash-stats-total = total
