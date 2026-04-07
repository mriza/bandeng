package main

import (
	"context"
	"crypto/sha256"
	"crypto/tls"
	"encoding/hex"
	"fmt"
	"log"
	"strings"
	"sync"
	"time"

	"github.com/go-routeros/routeros/v3"
)

// App struct
type App struct {
	ctx     context.Context
	client  *routeros.Client
	address string
	logs    []string
	mu      sync.Mutex
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

// Connect to Mikrotik router with SSL support
func (a *App) Connect(address, username, password string, secure bool, insecure bool) map[string]interface{} {
	a.mu.Lock()
	defer a.mu.Unlock()

	// Handle default ports
	if !strings.Contains(address, ":") {
		if secure {
			address += ":8729"
		} else {
			address += ":8728"
		}
	}
	a.address = address

	var client *routeros.Client
	var err error

	if secure {
		tlsConfig := &tls.Config{
			InsecureSkipVerify: insecure,
		}
		a.addLog(fmt.Sprintf("Attempting secure connection to %s (Skip Verify: %v)", address, insecure))
		client, err = routeros.DialTLS(address, username, password, tlsConfig)
		
		if err != nil && !insecure {
			// Check if it's a certificate validation error
			if isCertError(err) {
				a.addLog(fmt.Sprintf("Certificate validation failed: %v", err))
				certInfo := a.getCertificateInfo(address)
				if certInfo != nil {
					return map[string]interface{}{
						"status":    "cert_error",
						"cert_info": certInfo,
						"message":   err.Error(),
					}
				}
			}
		}
	} else {
		a.addLog(fmt.Sprintf("Attempting API connection to %s", address))
		client, err = routeros.Dial(address, username, password)
	}

	if err != nil {
		a.addLog(fmt.Sprintf("Connection failed to %s: %v", address, err))
		return map[string]interface{}{
			"status":  "error",
			"message": fmt.Sprintf("Connection failed: %v", err),
		}
	}

	// Disconnect old client if exists
	if a.client != nil {
		a.client.Close()
	}

	a.client = client
	a.addLog(fmt.Sprintf("Connected to %s as %s (Secure: %v)", address, username, secure))
	return map[string]interface{}{
		"status":  "success",
		"message": "Connected successfully",
	}
}

// isCertError checks if the error is related to certificate validation
func isCertError(err error) bool {
	msg := strings.ToLower(err.Error())
	return strings.Contains(msg, "certificate") || 
		   strings.Contains(msg, "x509") || 
		   strings.Contains(msg, "authority") ||
		   strings.Contains(msg, "hostname") ||
		   strings.Contains(msg, "signed by unknown")
}

// getCertificateInfo fetches the certificate details for user inspection
func (a *App) getCertificateInfo(address string) map[string]interface{} {
	conf := &tls.Config{
		InsecureSkipVerify: true,
	}

	// Create a short-lived connection just to peek at the cert
	dialer := &tls.Dialer{Config: conf}
	conn, err := dialer.DialContext(context.Background(), "tcp", address)
	if err != nil {
		return nil
	}
	defer conn.Close()

	tlsConn, ok := conn.(*tls.Conn)
	if !ok {
		return nil
	}

	state := tlsConn.ConnectionState()
	if len(state.PeerCertificates) == 0 {
		return nil
	}

	cert := state.PeerCertificates[0]
	fingerprint := sha256.Sum256(cert.Raw)
	
	return map[string]interface{}{
		"subject":     cert.Subject.String(),
		"issuer":      cert.Issuer.String(),
		"validFrom":   cert.NotBefore.Format("2006-01-02"),
		"validTo":     cert.NotAfter.Format("2006-01-02"),
		"fingerprint": strings.ToUpper(hex.EncodeToString(fingerprint[:])),
	}
}

// Disconnect from Mikrotik router
func (a *App) Disconnect() string {
	a.mu.Lock()
	defer a.mu.Unlock()
	
	if a.client != nil {
		a.client.Close()
		a.client = nil
		a.addLog("Disconnected from router")
	}
	return "Disconnected"
}

// Get IP bindings
func (a *App) GetIPBindings() map[string]interface{} {
	a.mu.Lock()
	defer a.mu.Unlock()

	if a.client == nil {
		a.addLog("GetIPBindings: Not connected")
		return map[string]interface{}{"error": "Not connected"}
	}

	reply, err := a.client.Run("/ip/hotspot/ip-binding/print", "=.proplist=.id,address,mac-address,type,comment,server")
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
	a.mu.Lock()
	defer a.mu.Unlock()

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
	a.mu.Lock()
	defer a.mu.Unlock()

	if a.client == nil {
		a.addLog("AddIPBinding: Not connected")
		return "Not connected"
	}

	a.addLog(fmt.Sprintf("Executing command: /ip/hotspot/ip-binding/add server=%s mac-address=%s type=%s comment=%s", server, mac, typ, comment))

	_, err := a.client.Run("/ip/hotspot/ip-binding/add", "=server="+server, "=mac-address="+mac, "=type="+typ, "=comment="+comment)
	if err != nil {
		a.addLog(fmt.Sprintf("AddIPBinding failed for MAC %s: %v", mac, err))
		return fmt.Sprintf("Failed to add binding: %v", err)
	}
	a.addLog(fmt.Sprintf("Added IP binding for MAC %s, server %s, type %s", mac, server, typ))
	return "Binding added successfully"
}

// Remove IP binding
func (a *App) RemoveIPBinding(id string) string {
	a.mu.Lock()
	defer a.mu.Unlock()

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

// Sync IP binding with ARP table
func (a *App) SyncIPBinding(id, mac string) string {
	a.mu.Lock()
	defer a.mu.Unlock()

	if a.client == nil {
		a.addLog("SyncIPBinding: Not connected")
		return "Not connected"
	}

	a.addLog(fmt.Sprintf("Searching ARP table for MAC: %s", mac))
	// Search in ARP table for specific MAC
	reply, err := a.client.Run("/ip/arp/print", "?mac-address="+mac)
	if err != nil {
		a.addLog(fmt.Sprintf("ARP search failed for MAC %s: %v", mac, err))
		return fmt.Sprintf("Search failed: %v", err)
	}

	if len(reply.Re) == 0 {
		a.addLog(fmt.Sprintf("MAC %s not found in ARP table", mac))
		return "Not found in ARP table"
	}

	// Get IP address from the first match
	ip := ""
	for _, re := range reply.Re {
		if val, ok := re.Map["address"]; ok {
			ip = val
			break
		}
	}

	if ip == "" {
		return "No IP found in ARP"
	}

	a.addLog(fmt.Sprintf("Found IP %s for MAC %s. Updating binding %s...", ip, mac, id))

	// Update the IP binding
	_, err = a.client.Run("/ip/hotspot/ip-binding/set", "=.id="+id, "=address="+ip)
	if err != nil {
		a.addLog(fmt.Sprintf("Failed to update binding %s: %v", id, err))
		return fmt.Sprintf("Failed to update: %v", err)
	}

	a.addLog(fmt.Sprintf("Successfully synchronized binding %s with IP %s", id, ip))
	return "Success: " + ip
}

// Sync all IP bindings with ARP table
func (a *App) SyncAllIPBindings() string {
	a.mu.Lock()
	defer a.mu.Unlock()

	if a.client == nil {
		a.addLog("SyncAllIPBindings: Not connected")
		return "Not connected"
	}

	a.addLog("Starting synchronization for all IP bindings...")
	
	// Get all IP bindings
	reply, err := a.client.Run("/ip/hotspot/ip-binding/print", "=.proplist=.id,address,mac-address")
	if err != nil {
		a.addLog(fmt.Sprintf("SyncAll failed to fetch bindings: %v", err))
		return fmt.Sprintf("Failed to fetch bindings: %v", err)
	}

	// Get ARP table
	arpReply, err := a.client.Run("/ip/arp/print", "=.proplist=address,mac-address")
	if err != nil {
		a.addLog(fmt.Sprintf("SyncAll failed to fetch ARP table: %v", err))
		return fmt.Sprintf("Failed to fetch ARP table: %v", err)
	}

	// Create ARP map for fast lookup
	arpMap := make(map[string]string)
	for _, re := range arpReply.Re {
		mac := re.Map["mac-address"]
		ip := re.Map["address"]
		if mac != "" && ip != "" {
			arpMap[mac] = ip
		}
	}

	count := 0
	for _, re := range reply.Re {
		id := re.Map[".id"]
		currentAddress := re.Map["address"]
		mac := re.Map["mac-address"]

		if mac != "" {
			if ip, found := arpMap[mac]; found {
				// Only update if current address is empty or different
				if currentAddress != ip {
					_, err = a.client.Run("/ip/hotspot/ip-binding/set", "=.id="+id, "=address="+ip)
					if err == nil {
						count++
					}
				}
			}
		}
	}

	a.addLog(fmt.Sprintf("Bulk synchronization completed. Updated %d bindings.", count))
	return fmt.Sprintf("Successfully synchronized %d bindings", count)
}

// Get logs
func (a *App) GetLogs() []string {
	a.mu.Lock()
	defer a.mu.Unlock()
	return a.logs
}

// Get system info
func (a *App) GetSystemInfo() map[string]interface{} {
	a.mu.Lock()
	defer a.mu.Unlock()

	if a.client == nil {
		a.addLog("GetSystemInfo: Not connected")
		return map[string]interface{}{"error": "Not connected"}
	}

	reply, err := a.client.Run("/system/identity/print")
	if err != nil {
		a.addLog(fmt.Sprintf("GetSystemInfo failed: %v", err))
		return map[string]interface{}{"error": fmt.Sprintf("Failed to get system info: %v", err)}
	}

	info := make(map[string]interface{})
	for _, re := range reply.Re {
		if name, ok := re.Map["name"]; ok {
			info["name"] = name
			break
		}
	}
	return info
}