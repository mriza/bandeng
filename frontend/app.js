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

    // Replace icons with local SVGs
    replaceIcons();
};

async function replaceIcons() {
    const icons = document.querySelectorAll('[data-lucide]');
    for (const icon of icons) {
        const name = icon.getAttribute('data-lucide');
        try {
            const response = await fetch(`icons/${name}.svg`);
            if (!response.ok) continue;
            const svgText = await response.text();
            
            const wrapper = document.createElement('div');
            wrapper.innerHTML = svgText.trim();
            const svg = wrapper.firstChild;
            
            // Transfer classes
            if (icon.className) {
                // Keep original classes
                svg.setAttribute('class', icon.className);
            }
            
            icon.parentNode.replaceChild(svg, icon);
        } catch (e) {
            console.error(`Failed to load icon ${name}`, e);
        }
    }
}

const { Connect, Disconnect, GetIPBindings, AddIPBinding, RemoveIPBinding, GetSystemInfo, GetLogs, GetHotspotServers } = window.go.main.App;

let currentBindings = [];
let currentPage = 1;
const itemsPerPage = 15;

function showStatus(message, type = 'info') {
    const container = document.getElementById('status-container');
    const alert = document.createElement('div');
    
    let alertClass = 'alert-info';
    if (type === 'error') alertClass = 'alert-error';
    if (type === 'success') alertClass = 'alert-success';

    alert.className = `alert ${alertClass} shadow-lg mb-2 transition-all duration-300 transform translate-y-4 opacity-0`;
    alert.innerHTML = `
        <div>
            <span>${message}</span>
        </div>
    `;
    
    container.appendChild(alert);
    
    // Trigger animation
    setTimeout(() => {
        alert.classList.remove('translate-y-4', 'opacity-0');
    }, 10);

    // Remove after 3 seconds
    setTimeout(() => {
        alert.classList.add('translate-y-4', 'opacity-0');
        setTimeout(() => alert.remove(), 300);
    }, 3000);
}

async function connect() {
    const address = document.getElementById('address').value;
    const username = document.getElementById('username').value;
    const password = document.getElementById('password').value;

    const result = await Connect(address, username, password);
    
    if (result === "Connected successfully") {
        showStatus(result, 'success');
        localStorage.setItem('routerAddress', address);
        localStorage.setItem('routerUsername', username);
        document.getElementById('login-container').classList.add('hidden');
        document.getElementById('main-container').classList.remove('hidden');
        loadSystemInfo();
        loadBindings();
        loadLogs();
        loadServers();
    } else {
        showStatus(result, 'error');
    }
}

async function disconnect() {
    const result = await Disconnect();
    showStatus(result, 'info');
    document.getElementById('login-container').classList.remove('hidden');
    document.getElementById('main-container').classList.add('hidden');
    document.getElementById('bindings-body').innerHTML = '';
}

async function loadLogs() {
    const logs = await GetLogs();
    const logsText = document.getElementById('logs-text');
    logsText.value = logs.join('\n');
}

async function loadSystemInfo() {
    const info = await GetSystemInfo();
    const infoEl = document.getElementById('sys-info');
    if (info.error) {
        infoEl.innerText = info.error;
    } else {
        infoEl.innerText = `Identity: ${info.name || 'MikroTik'}`;
    }
}

async function loadServers() {
    const servers = await GetHotspotServers();
    const select = document.getElementById('modal-server');
    select.innerHTML = '';
    
    // Add "all" option as default
    const allOption = document.createElement('option');
    allOption.value = 'all';
    allOption.text = 'all (Default)';
    select.appendChild(allOption);

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
        showStatus("Failed to load bindings: " + error, 'error');
        return;
    }

    if (!bindings || bindings.length === 0) {
        showStatus("No bindings found", 'info');
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

    pageBindings.forEach((binding, i) => {
        const row = tbody.insertRow();
        
        row.insertCell(0).innerHTML = `<span class="font-mono opacity-50 text-xs">${start + i + 1}</span>`;
        row.insertCell(1).innerHTML = `<span class="badge badge-ghost badge-sm font-mono">${binding['.id'] || ''}</span>`;
        row.insertCell(2).innerText = binding['ip-address'] || '-';
        row.insertCell(3).innerText = binding['mac-address'] || '';
        row.insertCell(4).innerText = binding.server || 'all';
        
        // Type Badge
        const typeCell = row.insertCell(5);
        let badgeType = 'badge-ghost';
        if (binding.type === 'bypassed') badgeType = 'badge-success';
        if (binding.type === 'blocked') badgeType = 'badge-error';
        typeCell.innerHTML = `<span class="badge ${badgeType} badge-sm capitalize">${binding.type || 'regular'}</span>`;
        
        row.insertCell(6).innerText = binding.comment || '';

        const actionsCell = row.insertCell(7);
        actionsCell.className = "text-center";
        const removeBtn = document.createElement('button');
        removeBtn.className = 'btn btn-ghost btn-xs text-error tooltip';
        removeBtn.setAttribute('data-tip', 'Remove Binding');
        removeBtn.innerHTML = '<i data-lucide="trash-2" class="w-4 h-4"></i>';
        removeBtn.onclick = () => removeBinding(binding['.id']);
        actionsCell.appendChild(removeBtn);
    });

    updatePaginationControls(totalPages);
    
    // Replace icons with local SVGs
    replaceIcons();
}

function updatePaginationControls(totalPages) {
    const paginationControls = document.getElementById('pagination-controls');
    if (totalPages <= 1) {
        paginationControls.classList.add('hidden');
        return;
    }
    
    paginationControls.classList.remove('hidden');
    
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
            errorSpan.innerText = 'Invalid MAC format (XX:XX:XX:XX:XX:XX)';
        } else {
            showStatus('Invalid MAC address format', 'error');
        }
        return;
    }

    const result = await AddIPBinding(mac, server, typ, comment);
    
    if (result === "Binding added successfully") {
        showStatus(result, 'success');
        closeModal();
        loadBindings();
        // Reset fields
        document.getElementById('modal-mac').value = '';
        document.getElementById('modal-server').selectedIndex = 0;
        document.getElementById('modal-comment').value = '';
    } else {
        showStatus(result, 'error');
    }
}

function openModal() {
    document.getElementById('addModal').showModal();
}

function closeModal() {
    document.getElementById('addModal').close();
}

async function removeBinding(id) {
    if (!confirm(`Are you sure you want to remove binding ID ${id}?`)) return;
    
    const result = await RemoveIPBinding(id);
    if (result === "Binding removed successfully") {
        showStatus(result, 'success');
        loadBindings();
    } else {
        showStatus(result, 'error');
    }
}