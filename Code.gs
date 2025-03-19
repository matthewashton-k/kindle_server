function doGet(e) {
  const params = e.parameter;
  const action = params.action;

  try {
    if (action === 'syncTasks') {
      const taskListId = params.taskListId;
      const tasks = JSON.parse(params.tasks);
      return handleSyncTasks(taskListId, tasks);
    } else if (action === 'getTaskLists') {
      const taskLists = getTaskLists();
      return createJsonResponse(taskLists);
    } else if (action === 'getTasks') {
      const taskListId = params.taskListId;
      const tasks = getTasks(taskListId);
      return createJsonResponse(tasks);
    } else {
      throw new Error('Invalid action');
    }
  } catch (error) {
    return createJsonResponse({ error: error.message });
  }
}

function handleSyncTasks(taskListId, tasks) {
  // Clear existing tasks
  const existingTasks = Tasks.Tasks.list(taskListId).items || [];
  existingTasks.forEach(task => {
    Tasks.Tasks.remove(taskListId, task.id);
  });

  // Add new tasks with completion status
  const results = tasks.map(task => {

    const newTask = Tasks.newTask()
      .setTitle(task.title)
      .setNotes(task.notes || '')
      .setStatus(task.completed ? 'completed' : 'needsAction');
    
    if (task.completed) {
      newTask.setCompleted(null);
    }
    
    return Tasks.Tasks.insert(newTask, taskListId);
  });

  return createJsonResponse({ success: true, count: results.length });
}

function doPost(e) {
  const params = e.parameter;
  const action = params.action;

  try {
    if (action === 'syncTasks') {
      const taskListId = params.taskListId;
      const tasks = JSON.parse(params.tasks);
      return handleSyncTasks(taskListId, tasks);
    } if (action === 'addTask') {
      const taskListId = params.taskListId;
      const title = params.title;
      addTask(taskListId, title);
      return createJsonResponse({ status: 'success' });
    } else if (action === 'setCompleted') {
      const taskListId = params.taskListId;
      const taskId = params.taskId;
      const completed = params.completed === 'true';
      setCompleted(taskListId, taskId, completed);
      return createJsonResponse({ status: 'success' });
    } else if (action === 'deleteTask') {
      const taskListId = params.taskListId;
      const taskId = params.taskId;
      Tasks.Tasks.remove(taskListId, taskId);
      return createJsonResponse({ status: 'success' });
    } else {
      throw new Error('Invalid action');
    }
  } catch (error) {
    return createJsonResponse({ error: error.message });
  }
}

function createJsonResponse(data) {
  return ContentService.createTextOutput(JSON.stringify(data))
    .setMimeType(ContentService.MimeType.JSON);
}



/**
 * Returns the ID and name of every task list in the user's account.
 * @return {Array.<Object>} The task list data.
 */
function getTaskLists() {
  var taskLists = Tasks.Tasklists.list().getItems();
  if (!taskLists) {
    return [];
  }
  return taskLists.map(function(taskList) {
    return {
      id: taskList.getId(),
      name: taskList.getTitle()
    };
  });
}

/**
 * Returns information about the tasks within a given task list.
 * @param {String} taskListId The ID of the task list.
 * @return {Array.<Object>} The task data.
 */
function getTasks(taskListId) {
  var tasks = Tasks.Tasks.list(taskListId).getItems();
  if (!tasks) {
    return [];
  }
  return tasks.map(function(task) {
    return {
      id: task.getId(),
      title: task.getTitle(),
      notes: task.getNotes(),
      completed: Boolean(task.getCompleted())
    };
  }).filter(function(task) {
    return task.title;
  });
}

/**
 * Sets the completed status of a given task.
 * @param {String} taskListId The ID of the task list.
 * @param {String} taskId The ID of the task.
 * @param {Boolean} completed True if the task should be marked as complete, false otherwise.
 */
function setCompleted(taskListId, taskId, completed) {
  var task = Tasks.newTask();
  if (completed) {
    task.setStatus('completed');
  } else {
    task.setStatus('needsAction');
    task.setCompleted(null);
  }
  Tasks.Tasks.patch(task, taskListId, taskId);
}

/**
 * Adds a new task to the task list.
 * @param {String} taskListId The ID of the task list.
 * @param {String} title The title of the new task.
 */
function addTask(taskListId, title) {
  var task = Tasks.newTask().setTitle(title);
  Tasks.Tasks.insert(task, taskListId);
}
