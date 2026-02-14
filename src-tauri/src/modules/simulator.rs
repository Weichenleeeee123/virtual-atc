use std::error::Error;

pub struct SimulatorConnection {
    // TODO: Add X-Plane or SimConnect connection
}

impl SimulatorConnection {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        // TODO: Implement actual connection logic
        // For now, return a mock connection
        Ok(SimulatorConnection {})
    }
    
    pub fn get_flight_data(&self) -> Result<FlightData, Box<dyn Error>> {
        // TODO: Implement actual data retrieval from simulator
        // For now, return mock data
        Ok(FlightData {
            callsign: "CCA123".to_string(),
            altitude: 5000.0,
            speed: 250.0,
            heading: 90.0,
        })
    }
}

#[derive(serde::Serialize, Clone)]
pub struct FlightData {
    pub callsign: String,
    pub altitude: f64,
    pub speed: f64,
    pub heading: f64,
}
