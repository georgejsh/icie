const vscode = acquireVsCodeApi();

let newing = false;

function make_action(callback) {
	return function() {
		vscode.postMessage({ "tag": "action_notice" });
		let action = event.target;
		let cell = action.parentElement.parentElement;
		let row = cell.parentElement;
		let path_in = row.dataset['path_in'];
		return callback({
			row: row,
			cell: cell,
			action: action,
			path_in: path_in
		});
	};
}

action_copy = make_action(ev => {
	let data_node = class_kid(ev.cell, ['data']);
	let selection = window.getSelection();
	let range = document.createRange();
	range.selectNodeContents(data_node);
	selection.removeAllRanges();
	selection.addRange(range);
	document.execCommand('Copy');
	selection.removeAllRanges();
});
action_rr = make_action(ev => vscode.postMessage({ tag: "trigger_rr", in_path: ev.path_in }));
action_gdb = make_action(ev => vscode.postMessage({ tag: "trigger_gdb", in_path: ev.path_in }));
action_setalt = make_action(ev => vscode.postMessage({ tag: "set_alt", in_path: ev.path_in, out: ev.row.dataset['raw_out'] }));
action_delalt = make_action(ev => vscode.postMessage({ tag: "del_alt", in_path: ev.path_in }));
action_edit = make_action(ev => {
	let path = ev.path_in;
	if (ev.cell.classList.contains("desired")) {
		path = path.replace(/\.in$/, '.out');
	}
	vscode.postMessage({ tag: "edit", path: path });
});

function new_start() {
	console.log(`new_start()`);
    vscode.postMessage({ tag: 'start_running'});
	if (!newing) {
		document.getElementsByClassName('new')[0].classList.add("is-active");
        document.getElementById("new-input-fixed").innerText="";
		newing = true;
	}
	for (let to_hide of document.getElementsByClassName('new-tutorial-start')) {
		to_hide.style.display = 'none';
	}
	document.getElementById('new-input').focus();
    document.getElementById('new_button').style.visibility = 'hidden';
    document.getElementById("new-error").disabled=true;
    document.getElementById('new-input').style.visibility = 'visible';
}

function new_confirm() {
	console.log(`new_confirm()`);
	let input = document.getElementById("new-input-fixed").innerText+document.getElementById('new-input').value;
	let desired = document.getElementById('new-desired').value;
	vscode.postMessage({
		tag: "new_test",
		input: input,
		desired: desired
	});
	new_shutdown();
}

function new_shutdown() {
	console.log(`new_shutdown()`);
	if (!newing) {
		throw new Error('shut down the test even though creation has not been started');
    }
	document.getElementsByClassName('new')[0].classList.remove('is-active');
	document.getElementById('new-input').value = '';
	document.getElementById('new-desired').value = '';
    document.getElementById('new-error').value = '';
    document.getElementById('new-input-fixed').value = '';
    document.getElementById('message').innerText = '';
    document.getElementById("new-input").disabled=false;
	newing = false;
    document.getElementById('new_button').style.visibility = 'visible';
    document.getElementById('new_button').classList.remove('is-active');
}

function scroll_to_wa() {
	let failed = document.getElementsByClassName('status-failed');
	if (failed.length > 0) {
		failed[0].scrollIntoView();
	} else {
		let ignore = document.getElementsByClassName('status-ignore');
		if (ignore.length > 0) {
			ignore[0].scrollIntoView();
		}
	}
}

window.addEventListener('message', event => {
	let message = event.data;
	if (message.tag === 'new_start') {
		if (!newing) {
			new_start();
		} else {
			new_confirm();
		}
	} else if (message.tag === 'scroll_to_wa') {
		scroll_to_wa();
	} else if (message.tag === 'eval_resp') {
		eval_finish(message);
	} else if (message.tag === 'output') {
		append_output(message);
	}
    else if (message.tag === 'error') {
		append_error(message);
	}
    else if (message.tag=== 'run_done'){
        process_exit(message)
    }
});

window.addEventListener('load', () => {
	for (let tx of document.getElementsByTagName('textarea')) {
		autoexpand_textarea(tx);
	}
	for (let row of Array.from(document.getElementsByClassName('row'))) {
		let output = class_kid(row, ['output', 'data']);
		let desired = class_kid(row, ['desired', 'data']);
		sync_scroll(output, desired);
	}
    
}, false);

let cursor_x = 0;
let cursor_y = 0;
window.addEventListener('mousemove', e => {
	cursor_x = e.clientX;
	cursor_y = e.clientY;
});
window.addEventListener('copy', e => {
	let data = window.getSelection().toString();
	if (data.trim() !== '') {
		return;
	}
	let element = document.elementFromPoint(cursor_x, cursor_y);
	while (element !== null && !element.classList.contains('cell')) {
		element = element.parentElement;
	}
	if (element === null) {
		return;
	}
	let text = element.dataset.raw;
	e.clipboardData.setData('text/plain', text);
	e.preventDefault();
});
let last_eval_id = 0;
window.onload = () => {
	let new_input = document.getElementById('new-input');
	new_input.addEventListener('blur', event => {
		let input = new_input.value.trim();
		let eval_id = ++last_eval_id;
		if (input !== '') {
			vscode.postMessage({ tag: 'eval_req', input: input, id: eval_id });
		}
	});
	vscode.postMessage({
		tag: "after_load"
	});
    document.getElementById("new-input").addEventListener("keypress", onTestChange)
};

function eval_finish(msg) {
	let new_input = document.getElementById('new-input');
	if (msg.id === last_eval_id && newing && new_input.value.trim() === msg.input) {
		new_shutdown();
	}
}

function autoexpand_textarea(tx) {
	tx.setAttribute('style', `height: ${Math.max(86, tx.scrollHeight)}px; overflow-y: hidden;`);
	tx.addEventListener('input', function () {
		this.style.height = 'auto';
        this.style.overflowY = 'hidden'
		this.style.height = `${Math.max(86, this.scrollHeight)}px`;
	}, false);
    tx.addEventListener('update', function () {
		this.style.height = 'auto';
        this.style.overflowY = 'hidden';
		this.style.height = `${Math.max(86, this.scrollHeight)}px`;
	}, false);
    
}

function sync_scroll(a, b) {
	let ma = 0;
	let mb = 0;
	a.onscroll = () => {
		if (mb === 0) {
			++ma;
			b.scrollTop = a.scrollTop;
			b.scrollLeft = a.scrollLeft;
		} else {
			--mb;
		}
	};
	b.onscroll = () => {
		if (ma === 0) {
			++mb;
			a.scrollTop = b.scrollTop;
			a.scrollLeft = b.scrollLeft;
		} else {
			--ma;
		}
	};
}

function class_kid(v, clss) {
	for (let cls of clss) {
		v = Array.from(v.children).find(u => u.classList.contains(cls));
	}
	return v;
}

function onTestChange(event) {
    var keyCode = event.hasOwnProperty('which') ? event.which : event.keyCode;

    // If the user has pressed enter
    if (keyCode === 13) {
        document.getElementById("new-input-fixed").innerText+=document.getElementById("new-input").value+"\n";
        var lines=document.getElementById("new-input").value;
        vscode.postMessage({ tag: "input_data", data: lines+"\n" });
        console.log("input " +lines);
        document.getElementById("new-input").value="";
        event.preventDefault();
        return false;
    }
    return true;
}

function append_output(msg) {
	let new_input = document.getElementById('new-desired');
	new_input.value+=msg.input
}

function append_error(msg) {
	let new_input = document.getElementById('new-error');
	new_input.value+=msg.input
}

function process_exit(msg) {
    if(!newing) return;
	let new_input = document.getElementById('message');
	new_input.innerText="Process exited with exit code:"+msg.input;
    document.getElementById("new-input").value="";
    document.getElementById("new-input").disabled=true;
    document.getElementById('new-input').style.visibility = 'hidden';
    document.getElementById('new-input').classList.remove('is-active');
}

