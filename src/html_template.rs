pub const HTML_TEMPLATE: &str = r#"
<!DOCTYPE html>
<html>
<head>
    <title>ProxyCat Configuration</title>
    <style>
        body { 
            font-family: Arial, sans-serif; 
            margin: 20px;
            display: flex;
            flex-direction: column;
            height: 100vh;
        }
        .container {
            display: flex;
            flex: 1;
            gap: 20px;
            margin-top: 20px;
        }
        .left-pane, .right-pane {
            flex: 1;
            padding: 20px;
            border: 1px solid #ccc;
            border-radius: 4px;
            overflow-y: auto;
        }
        .section { 
            margin-bottom: 20px; 
        }
        .list { 
            border: 1px solid #ddd; 
            padding: 10px; 
            min-height: 50px;
            background: #f9f9f9;
            border-radius: 4px;
        }
        .item { 
            background: #fff; 
            margin: 5px; 
            padding: 10px; 
            cursor: move;
            display: flex;
            align-items: center;
            border: 1px solid #eee;
            border-radius: 4px;
            transition: background-color 0.2s;
        }
        .item:hover {
            background: #f0f0f0;
        }
        .item input[type="checkbox"] { 
            margin-right: 10px; 
        }
        .pac-preview {
            font-family: monospace;
            white-space: pre-wrap;
            padding: 15px;
            border-radius: 4px;
            overflow-x: auto;
        }
        .pac-preview .keyword { color: #569cd6; }
        .pac-preview .string { color: #ce9178; }
        .pac-preview .comment { color: #6a9955; }
        .pac-preview .function { color: #dcdcaa; }
        .header {
            margin-bottom: 20px;
        }
        .add-form {
            margin-top: 10px;
            padding: 10px;
            background: #f5f5f5;
            border-radius: 4px;
        }
        .add-form input, .add-form button {
            margin: 5px;
            padding: 5px;
        }
        .add-form button {
            background: #4CAF50;
            color: white;
            border: none;
            padding: 8px 16px;
            border-radius: 4px;
            cursor: pointer;
        }
        .add-form button:hover {
            background: #45a049;
        }
        .add-button {
            background: #4CAF50;
            color: white;
            border: none;
            padding: 8px 16px;
            border-radius: 4px;
            cursor: pointer;
            margin-bottom: 10px;
        }
        .add-button:hover {
            background: #45a049;
        }
        .form-row {
            display: flex;
            gap: 10px;
            margin-bottom: 5px;
        }
        .form-row input {
            flex: 1;
        }
    </style>
</head>
<body>
    <div class="header">
        <h1>ProxyCat Configuration</h1>
    </div>
    
    <div class="container">
        <div class="left-pane">
            <div class="section">
                <h2>Proxy Rules</h2>
                <button class="add-button" onclick="showAddForm('proxyRules')">Add New Proxy Rule</button>
                <div id="proxyRules" class="list"></div>
                <div id="proxyRulesForm" class="add-form" style="display: none;">
                    <div class="form-row">
                        <input type="text" id="proxyRuleHost" placeholder="Host (e.g. * or example.com)">
                    </div>
                    <div class="form-row">
                        <input type="text" id="proxyRuleProxyHost" placeholder="Proxy Host">
                    </div>
                    <div class="form-row">
                        <input type="number" id="proxyRulePort" placeholder="Proxy Port">
                    </div>
                    <button onclick="addProxyRule()">Add</button>
                    <button onclick="hideAddForm('proxyRules')">Cancel</button>
                </div>
            </div>
            <div class="section">
                <h2>Bypass List</h2>
                <button class="add-button" onclick="showAddForm('bypassList')">Add New Bypass Rule</button>
                <div id="bypassList" class="list"></div>
                <div id="bypassListForm" class="add-form" style="display: none;">
                    <div class="form-row">
                        <input type="text" id="bypassHost" placeholder="Host to bypass">
                    </div>
                    <button onclick="addBypassRule()">Add</button>
                    <button onclick="hideAddForm('bypassList')">Cancel</button>
                </div>
            </div>
            <div class="section">
                <h2>External PAC Functions</h2>
                <button class="add-button" onclick="showAddForm('externalPacFunctions')">Add New PAC URL</button>
                <div id="externalPacFunctions" class="list"></div>
                <div id="externalPacFunctionsForm" class="add-form" style="display: none;">
                    <div class="form-row">
                        <input type="text" id="pacUrl" placeholder="PAC file URL">
                    </div>
                    <button onclick="addPacUrl()">Add</button>
                    <button onclick="hideAddForm('externalPacFunctions')">Cancel</button>
                </div>
            </div>
        </div>
        
        <div class="right-pane">
            <h2>PAC Configuration Preview</h2>
            <div id="pacPreview" class="pac-preview"></div>
        </div>
    </div>

    <script>
        console.log("Script starting...");

        // Get the current port from the server
        const currentPort = window.location.port;

        function showAddForm(formId) {
            document.getElementById(formId + 'Form').style.display = 'block';
        }

        function hideAddForm(formId) {
            document.getElementById(formId + 'Form').style.display = 'none';
        }

        async function addProxyRule() {
            const host = document.getElementById('proxyRuleHost').value;
            const proxyHost = document.getElementById('proxyRuleProxyHost').value;
            const port = parseInt(document.getElementById('proxyRulePort').value);

            if (!host || !proxyHost || !port) {
                alert('Please fill in all fields');
                return;
            }

            const item = {
                rule: {
                    host: host,
                    proxy_host: proxyHost,
                    proxy_port: port
                },
                enabled: true
            };

            try {
                const response = await fetch(`http://127.0.0.1:${currentPort}/add-item`, {
                    method: 'POST',
                    headers: {
                        'Content-Type': 'application/json',
                    },
                    body: JSON.stringify({
                        list_type: 'proxy_rules',
                        item: item
                    })
                });

                if (!response.ok) {
                    throw new Error(`HTTP error! status: ${response.status}`);
                }

                hideAddForm('proxyRules');
                document.getElementById('proxyRuleHost').value = '';
                document.getElementById('proxyRuleProxyHost').value = '';
                document.getElementById('proxyRulePort').value = '';
                loadConfig();
            } catch (error) {
                console.error('Error adding proxy rule:', error);
                alert('Failed to add proxy rule');
            }
        }

        async function addBypassRule() {
            const host = document.getElementById('bypassHost').value;

            if (!host) {
                alert('Please enter a host');
                return;
            }

            const item = {
                host: host,
                enabled: true
            };

            try {
                const response = await fetch(`http://127.0.0.1:${currentPort}/add-item`, {
                    method: 'POST',
                    headers: {
                        'Content-Type': 'application/json',
                    },
                    body: JSON.stringify({
                        list_type: 'bypass_list',
                        item: item
                    })
                });

                if (!response.ok) {
                    throw new Error(`HTTP error! status: ${response.status}`);
                }

                hideAddForm('bypassList');
                document.getElementById('bypassHost').value = '';
                loadConfig();
            } catch (error) {
                console.error('Error adding bypass rule:', error);
                alert('Failed to add bypass rule');
            }
        }

        async function addPacUrl() {
            const url = document.getElementById('pacUrl').value;

            if (!url) {
                alert('Please enter a URL');
                return;
            }

            try {
                const response = await fetch(`http://127.0.0.1:${currentPort}/add-item`, {
                    method: 'POST',
                    headers: {
                        'Content-Type': 'application/json',
                    },
                    body: JSON.stringify({
                        list_type: 'external_pac_functions',
                        item: {
                            function: {
                                original_url: url,
                                function_name: 'FindProxyForURL_' + url.replace(/[^a-zA-Z0-9]/g, '_'),
                                function_text: ''
                            },
                            enabled: true
                        }
                    })
                });

                if (!response.ok) {
                    throw new Error(`HTTP error! status: ${response.status}`);
                }

                hideAddForm('externalPacFunctions');
                document.getElementById('pacUrl').value = '';
                loadConfig();
            } catch (error) {
                console.error('Error adding PAC URL:', error);
                alert('Failed to add PAC URL');
            }
        }

        async function loadConfig() {
            try {
                console.log("Fetching config...");
                const response = await fetch(`http://127.0.0.1:${currentPort}/config`);
                if (!response.ok) {
                    throw new Error(`HTTP error! status: ${response.status}`);
                }
                const config = await response.json();
                console.log("Received config:", config);
                updateLists(config);
                updatePacPreview(config);
            } catch (error) {
                console.error("Error loading config:", error);
            }
        }

        function updateLists(config) {
            console.log("Updating lists with config:", config);
            if (!config) {
                console.error("No config data received");
                return;
            }

            updateList("proxyRules", config.proxy_rules);
            updateList("bypassList", config.bypass_list);
            updateList("externalPacFunctions", config.external_pac_functions);
        }

        function updateList(listId, items) {
            console.log(`Updating ${listId} with items:`, items);
            const list = document.getElementById(listId);
            if (!list) {
                console.error(`List element ${listId} not found`);
                return;
            }
            list.innerHTML = "";

            if (!items || !Array.isArray(items)) {
                console.error(`No items or invalid items for ${listId}`);
                return;
            }

            items.forEach((item, index) => {
                const div = document.createElement("div");
                div.className = "item";
                div.draggable = true;
                
                const checkbox = document.createElement("input");
                checkbox.type = "checkbox";
                checkbox.checked = item.enabled;
                checkbox.addEventListener("change", async () => {
                    try {
                        const response = await fetch(`http://127.0.0.1:${currentPort}/toggle/${listId}/${index}`, {
                            method: "POST"
                        });
                        if (!response.ok) {
                            throw new Error(`HTTP error! status: ${response.status}`);
                        }
                        loadConfig();
                    } catch (error) {
                        console.error("Error toggling item:", error);
                    }
                });

                let text;
                switch(listId) {
                    case "proxyRules":
                        text = `${item.rule.host} -> ${item.rule.proxy_host}:${item.rule.proxy_port}`;
                        break;
                    case "bypassList":
                        text = item.host;
                        break;
                    case "externalPacFunctions":
                        text = `${item.function.function_name} (${item.function.original_url})`;
                        break;
                    default:
                        text = "Unknown item type";
                }

                const textNode = document.createTextNode(text);
                div.appendChild(checkbox);
                div.appendChild(textNode);

                div.addEventListener("dragstart", (e) => {
                    e.dataTransfer.setData("text/plain", index.toString());
                });

                div.addEventListener("dragover", (e) => {
                    e.preventDefault();
                });

                div.addEventListener("drop", async (e) => {
                    e.preventDefault();
                    const fromIndex = parseInt(e.dataTransfer.getData("text/plain"));
                    const toIndex = index;
                    if (fromIndex === toIndex) return;

                    try {
                        const response = await fetch(`http://127.0.0.1:${currentPort}/move/${listId}/${fromIndex}/${toIndex}`, {
                            method: "POST"
                        });
                        if (!response.ok) {
                            throw new Error(`HTTP error! status: ${response.status}`);
                        }
                        loadConfig();
                    } catch (error) {
                        console.error("Error moving item:", error);
                    }
                });

                list.appendChild(div);
            });
        }

        async function updatePacPreview(config) {
            try {
                const response = await fetch("/pac-content");
                if (!response.ok) {
                    throw new Error("Failed to fetch PAC content");
                }
                const content = await response.text();
                const pacPreview = document.getElementById("pacPreview");
                pacPreview.textContent = content;
            } catch (error) {
                console.error("Error updating PAC preview:", error);
            }
        }

        // Initial load
        console.log("Performing initial load...");
        loadConfig();

        // Refresh every 5 seconds
        setInterval(loadConfig, 5000);
    </script>
</body>
</html>
"#;
