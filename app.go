package main

import (
	"context"
	"fmt"
	"log"
	"strings"
	"time"

	"github.com/go-routeros/routeros/v3"
)

// App struct
type App struct {
	ctx     context.Context
	client  *routeros.Client
	address string
	logs    []string
}

// NewApp creates a new App application struct
func NewApp() *App {
	return &App{logs: []string{}}
}

// addLog adds a log message
func (a *App) addLog(message string) {
	timestamp := time.Now().Format("2006-01-02 15:04:05")
	logMsg := fmt.Sprintf("[%s] %s", timestamp, message)
	a.logs = append(a.logs, logMsg)
	log.Println(logMsg) // Also print to console
}

// startup is called when the app starts. The context is saved
// so we can call the runtime methods
func (a *App) startup(ctx context.Context) {
	a.ctx = ctx
}

// Connect to Mikrotik router
func (a *App) Connect(address, username, password string) string {
	// Default port to 8728 if not specified
	if !strings.Contains(address, ":") {
		address += ":8728"
	}
	a.address = address
	client, err := routeros.Dial(address, username, password)
	if err != nil {
		a.addLog(fmt.Sprintf("Connection failed to %s: %v", address, err))
		return fmt.Sprintf("Connection failed: %v", err)
	}
	a.client = client
	a.addLog(fmt.Sprintf("Connected to %s as %s", address, username))
	return "Connected successfully"
}

// Disconnect from Mikrotik router
func (a *App) Disconnect() string {
	if a.client != nil {
		a.client.Close()
		a.client = nil
		a.addLog("Disconnected from router")
	}
	return "Disconnected"
}

// Get IP bindings
func (a *App) GetIPBindings() map[string]interface{} {
	if a.client == nil {
		a.addLog("GetIPBindings: Not connected")
		return map[string]interface{}{"error": "Not connected"}
	}

	reply, err := a.client.Run("/ip/hotspot/ip-binding/print", "=.proplist=.id,ip-address,mac-address,type,comment,server")
	if err != nil {
		a.addLog(fmt.Sprintf("GetIPBindings failed: %v", err))
		return map[string]interface{}{"error": fmt.Sprintf("Failed to get bindings: %v", err)}
	}

	var bindings []map[string]string
	for _, re := range reply.Re {
		binding := make(map[string]string)
		for key, value := range re.Map {
			binding[key] = value
		}
		bindings = append(bindings, binding)
	}
	a.addLog(fmt.Sprintf("Loaded %d IP bindings", len(bindings)))
	return map[string]interface{}{"bindings": bindings}
}

// Get hotspot servers
func (a *App) GetHotspotServers() []string {
	if a.client == nil {
		a.addLog("GetHotspotServers: Not connected")
		return []string{}
	}

	reply, err := a.client.Run("/ip/hotspot/print")
	if err != nil {
		a.addLog(fmt.Sprintf("GetHotspotServers failed: %v", err))
		return []string{}
	}

	var servers []string
	for _, re := range reply.Re {
		if name, ok := re.Map["name"]; ok {
			servers = append(servers, name)
		}
	}
	a.addLog(fmt.Sprintf("Loaded %d hotspot servers", len(servers)))
	return servers
}
func (a *App) AddIPBinding(mac, server, typ, comment string) string {
	if a.client == nil {
		a.addLog("AddIPBinding: Not connected")
		return "Not connected"
	}

	// Check if hotspot is configured
	_, err := a.client.Run("/ip/hotspot/print")
	if err != nil {
		a.addLog(fmt.Sprintf("Hotspot not configured: %v", err))
		return "Hotspot not configured on router"
	}

	// Check API write permissions
	_, err = a.client.Run("/system/identity/set name=test")
	if err == nil {
		// Revert if successful
		a.client.Run("/system/identity/set name=")
	} else {
		a.addLog(fmt.Sprintf("API write access denied: %v", err))
		return "API does not have write permissions"
	}

	a.addLog(fmt.Sprintf("Executing command: /ip/hotspot/ip-binding/add server=%s mac-address=%s type=%s comment=%s", server, mac, typ, comment))
	_, err = a.client.Run("/ip/hotspot/ip-binding/add", "=server="+server, "=mac-address="+mac, "=type="+typ, "=comment="+comment)
	if err != nil {
		a.addLog(fmt.Sprintf("AddIPBinding failed for MAC %s: %v", mac, err))
		return fmt.Sprintf("Failed to add binding: %v", err)
	}
	a.addLog(fmt.Sprintf("Added IP binding for MAC %s, server %s, type %s", mac, server, typ))
	return "Binding added successfully"
}

// Remove IP binding
func (a *App) RemoveIPBinding(id string) string {
	if a.client == nil {
		a.addLog("RemoveIPBinding: Not connected")
		return "Not connected"
	}

	_, err := a.client.Run("/ip/hotspot/ip-binding/remove", "=.id="+id)
	if err != nil {
		a.addLog(fmt.Sprintf("RemoveIPBinding failed for ID %s: %v", id, err))
		return fmt.Sprintf("Failed to remove binding: %v", err)
	}
	a.addLog(fmt.Sprintf("Removed IP binding ID %s", id))
	return "Binding removed successfully"
}

// Get logs
func (a *App) GetLogs() []string {
	return a.logs
}