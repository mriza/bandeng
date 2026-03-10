// Load saved connection details on page load
window.onload = function() {
    const savedAddress = localStorage.getItem('routerAddress');
    const savedUsername = localStorage.getItem('routerUsername');
    if (savedAddress) document.getElementById('address').value = savedAddress;
    if (savedUsername) document.getElementById('username').value = savedUsername;

    // Pagination event listeners
    document.getElementById('prev-page').addEventListener('click', () => {
        if (currentPage > 1) {
            renderPage(currentPage - 1);
        }
    });
    document.getElementById('next-page').addEventListener('click', () => {
        const totalPages = Math.ceil(currentBindings.length / itemsPerPage);
        if (currentPage < totalPages) {
            renderPage(currentPage + 1);
        }
    });

    // MAC Address formatting and validation
    document.getElementById('modal-mac').addEventListener('input', function(e) {
        let val = e.target.value.replace(/[^A-Fa-f0-9]/g, '').toUpperCase();
        let formatted = '';
        for (let i = 0; i < val.length; i++) {
            if (i > 0 && i % 2 === 0) formatted += ':';
            formatted += val[i];
        }
        e.target.value = formatted.substring(0, 17);
        const errorSpan = document.getElementById('mac-error');
        if (errorSpan) errorSpan.innerText = '';
    });
};

const { Connect, Disconnect, GetIPBindings, AddIPBinding, RemoveIPBinding, GetSystemInfo, GetLogs, GetHotspotServers } = window.go.main.App;;

let currentBindings = [];
let currentPage = 1;
const itemsPerPage = 15;

function showStatus(message) {
    document.getElementById('status').innerText = message;
}

async function connect() {
    const address = document.getElementById('address').value;
    const username = document.getElementById('username').value;
    const password = document.getElementById('password').value;

    const result = await Connect(address, username, password);
    showStatus(result);

    if (result === "Connected successfully") {
        localStorage.setItem('routerAddress', address);
        localStorage.setItem('routerUsername', username);
        document.getElementById('login-container').style.display = 'none';
        document.getElementById('main-container').style.display = 'flex';
        loadSystemInfo();
        document.getElementById('bindings').style.display = 'block';
        document.getElementById('logs').style.display = 'block';
        loadBindings(); // Load bindings automatically
        loadLogs(); // Load logs
        loadServers(); // Load servers for dropdown
    }
}

async function disconnect() {
    const result = await Disconnect();
    showStatus(result);
    document.getElementById('login-container').style.display = 'flex';
    document.getElementById('main-container').style.display = 'none';
    document.getElementById('bindings-body').innerHTML = '';
}

async function loadLogs() {
    const logs = await GetLogs();
    const logsText = document.getElementById('logs-text');
    logsText.value = logs.join('\n');
}

async function loadSystemInfo() {
    const info = await GetSystemInfo();
    if (info.error) {
        document.getElementById('sys-info').innerText = info.error;
    } else {
        document.getElementById('sys-info').innerText = `Router Name: ${info.name || 'Unknown'}`;
    }
}

async function loadServers() {
    const servers = await GetHotspotServers();
    const select = document.getElementById('modal-server');
    select.innerHTML = ''; // Clear existing options
    servers.forEach(server => {
        const option = document.createElement('option');
        option.value = server;
        option.text = server;
        select.appendChild(option);
    });
}
async function loadBindings() {
    const result = await GetIPBindings();
    const bindings = result.bindings;
    const error = result.error;

    if (error) {
        showStatus("Failed to load bindings: " + error);
        return;
    }

    if (!bindings || bindings.length === 0) {
        showStatus("No bindings found");
        currentBindings = [];
        renderPage(1);
        return;
    }

    currentBindings = bindings;
    currentPage = 1;
    renderPage(currentPage);
}

function renderPage(page) {
    const tbody = document.getElementById('bindings-body');
    tbody.innerHTML = '';

    const totalPages = Math.ceil(currentBindings.length / itemsPerPage);
    if (page < 1) page = 1;
    if (page > totalPages) page = totalPages;
    currentPage = page;

    const start = (page - 1) * itemsPerPage;
    const end = start + itemsPerPage;
    const pageBindings = currentBindings.slice(start, end);

    pageBindings.forEach(binding => {
        const row = tbody.insertRow();
        row.insertCell(0).innerText = binding['.id'] || '';
        row.insertCell(1).innerText = binding['ip-address'] || '';
        row.insertCell(2).innerText = binding['mac-address'] || '';
        row.insertCell(3).innerText = binding.type || '';
        row.insertCell(4).innerText = binding.comment || '';

        const actionsCell = row.insertCell(5);
        const removeBtn = document.createElement('button');
        removeBtn.innerText = 'Remove';
        removeBtn.onclick = () => removeBinding(binding['.id']);
        actionsCell.appendChild(removeBtn);
    });

    updatePaginationControls(totalPages);
}

function updatePaginationControls(totalPages) {
    const paginationControls = document.getElementById('pagination-controls');
    if (totalPages <= 1) {
        paginationControls.style.display = 'none';
        return;
    }
    
    paginationControls.style.display = 'block';
    
    const prevBtn = document.getElementById('prev-page');
    const nextBtn = document.getElementById('next-page');
    const pageInfo = document.getElementById('page-info');

    prevBtn.disabled = currentPage <= 1;
    nextBtn.disabled = currentPage >= totalPages;

    pageInfo.innerText = `Page ${currentPage} of ${totalPages}`;
}

async function addBinding() {
    const mac = document.getElementById('modal-mac').value;
    const server = document.getElementById('modal-server').value;
    const typ = document.getElementById('modal-type').value;
    const comment = document.getElementById('modal-comment').value;

    const macRegex = /^([0-9A-Fa-f]{2}[:-]){5}([0-9A-Fa-f]{2})$/;
    if (!macRegex.test(mac)) {
        const errorSpan = document.getElementById('mac-error');
        if (errorSpan) {
            errorSpan.innerText = 'Invalid MAC Address format. Must be 12 hex characters.';
        } else {
            alert('Invalid MAC Address format.');
        }
        return;
    }

    const result = await AddIPBinding(mac, server, typ, comment);
    showStatus(result);
    if (result === "Binding added successfully") {
        closeModal();
        loadBindings(); // Refresh bindings
        document.getElementById('modal-mac').value = '';
        document.getElementById('modal-server').value = '';
        document.getElementById('modal-comment').value = '';
    }
}

function openModal() {
    document.getElementById('addModal').style.display = 'block';
}

function closeModal() {
    document.getElementById('addModal').style.display = 'none';
}

async function removeBinding(id) {
    const result = await RemoveIPBinding(id);
    showStatus(result);
    if (result === "Binding removed successfully") {
        loadBindings();
    }
}