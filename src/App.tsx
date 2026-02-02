import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

interface Service {
  name: String;
  ip: String;
  port: String;
}

function App() {
  const [services, setServices] = useState<Service[]>([]);
  const [name, setName] = useState("");
  const [ip, setIp] = useState("");
  const [port, setPort] = useState("");
  const [interval, setIntervalVal] = useState<number>(10);
  const [iconSet, setIconSet] = useState<string>("default");
  
  // Track which index we are editing. -1 means adding new.
  const [editIndex, setEditIndex] = useState<number>(-1);

  const fetchData = async () => {
    try {
      const list = await invoke<Service[]>("list_services");
      setServices(list);
      const currentInterval = await invoke<number>("get_interval");
      setIntervalVal(currentInterval);
      const currentIconSet = await invoke<string>("get_icon_set");
      setIconSet(currentIconSet);
    } catch (error) {
      console.error("Failed to fetch data:", error);
    }
  };

  useEffect(() => {
    fetchData();
  }, []);

  const handleIntervalChange = async (e: React.ChangeEvent<HTMLSelectElement>) => {
      const newInterval = parseInt(e.target.value);
      setIntervalVal(newInterval);
      try {
          await invoke("set_interval", { interval: newInterval });
      } catch (error) {
          console.error("Failed to set interval", error);
      }
  };

  const handleIconSetChange = async (e: React.ChangeEvent<HTMLSelectElement>) => {
      const newSet = e.target.value;
      console.log("Frontend: Switching icon set to:", newSet);
      setIconSet(newSet);
      try {
          console.log("Frontend: Invoking set_icon_set...");
          await invoke("set_icon_set", { preference: newSet });
          console.log("Frontend: Invoke success.");
      } catch (error) {
          console.error("Frontend: Failed to set icon set", error);
          alert("Error setting icons: " + JSON.stringify(error));
      }
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!name || !ip || !port) return;

    try {
      let updated: Service[];
      if (editIndex === -1) {
        // Add Mode
        updated = await invoke<Service[]>("add_service", {
          name,
          ip,
          port,
        });
      } else {
        // Edit Mode
        updated = await invoke<Service[]>("update_service", {
          index: editIndex,
          name,
          ip,
          port,
        });
        setEditIndex(-1); // Switch back to add mode
      }
      
      setServices(updated);
      setName("");
      setIp("");
      setPort("");
    } catch (error) {
      console.error("Failed to save service:", error);
      alert("Failed to save service: " + error);
    }
  };

  const handleEdit = (index: number) => {
    const svc = services[index];
    setName(svc.name as string);
    setIp(svc.ip as string);
    setPort(svc.port as string);
    setEditIndex(index);
  };

  const handleRemove = async (index: number) => {
    try {
      const updated = await invoke<Service[]>("remove_service", { index });
      setServices(updated);
      // If we were editing this one, cancel edit
      if (editIndex === index) {
        setEditIndex(-1);
        setName("");
        setIp("");
        setPort("");
      }
    } catch (error) {
      console.error("Failed to remove service:", error);
    }
  };

  const cancelEdit = () => {
    setEditIndex(-1);
    setName("");
    setIp("");
    setPort("");
  };

  return (
    <main className="container">
      <h1>Uptime Watcher Services</h1>

      <div className="settings-bar glass-panel">
          <div>
            <label>Check Interval: </label>
            <select value={interval} onChange={handleIntervalChange}>
                <option value={10}>10 Seconds</option>
                <option value={60}>1 Minute</option>
                <option value={600}>10 Minutes</option>
                <option value={1800}>30 Minutes</option>
                <option value={3600}>1 Hour</option>
            </select>
          </div>
          <div>
            <label>Icons: </label>
            <select value={iconSet} onChange={handleIconSetChange}>
                <option value="default">Default (Green/Red)</option>
                <option value="alt">Alternate (Check/Cross)</option>
            </select>
          </div>
      </div>

      <div className="service-list">
        {services.length === 0 ? (
          <p style={{ textAlign: "center", color: "#888" }}>No services monitored.</p>
        ) : (
          services.map((svc, idx) => (
            <div key={idx} className="service-item">
              <div className="service-info">
                <span className="service-name">{svc.name}</span>
                <span className="service-address">
                  {svc.ip}:{svc.port}
                </span>
              </div>
              <div className="actions">
                <button className="edit-btn" onClick={() => handleEdit(idx)}>
                  Edit
                </button>
                <button className="delete-btn" onClick={() => handleRemove(idx)}>
                  Remove
                </button>
              </div>
            </div>
          ))
        )}
      </div>

      <form className="add-form glass-panel" onSubmit={handleSubmit}>
        <h3>{editIndex === -1 ? "Add New Service" : "Edit Service"}</h3>
        
        <input
          placeholder="Name (e.g. Google)"
          value={name}
          onChange={(e) => setName(e.target.value)}
        />

        <div className="form-row">
            <input
            className="input-group-ip"
            placeholder="IP (e.g. 8.8.8.8)"
            value={ip}
            onChange={(e) => setIp(e.target.value)}
            />
            <input
            className="input-group-port"
            placeholder="Port (e.g. 53)"
            value={port}
            onChange={(e) => setPort(e.target.value)}
            />
        </div>
        <div className="form-actions">
            <button type="submit" className={editIndex === -1 ? "add-btn" : "update-btn"}>
            {editIndex === -1 ? "Add Service" : "Update Service"}
            </button>
            {editIndex !== -1 && (
                <button type="button" className="cancel-btn" onClick={cancelEdit}>
                    Cancel
                </button>
            )}
        </div>
      </form>
    </main>
  );
}

export default App;
