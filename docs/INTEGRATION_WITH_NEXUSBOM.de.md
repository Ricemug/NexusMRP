# 🔗 Integrationsleitfaden: NexusMRP + NexusBom

> Vollständiger Leitfaden zur Integration von Materialbedarfsplanung und Stückliste

[English](./INTEGRATION_WITH_NEXUSBOM.md) | [繁體中文](./INTEGRATION_WITH_NEXUSBOM.zh-TW.md) | [简体中文](./INTEGRATION_WITH_NEXUSBOM.zh-CN.md)

Dieser Leitfaden erklärt, wie Sie **NexusMRP** (Materialbedarfsplanung) mit **NexusBom** (Stückliste) integrieren, um ein vollständiges Fertigungsplanungssystem aufzubauen.

## 📋 Inhaltsverzeichnis

- [Überblick](#überblick)
- [Warum integrieren?](#warum-integrieren)
- [Architektur](#architektur)
- [Integrationsschritte](#integrationsschritte)
- [Codebeispiele](#codebeispiele)
- [Best Practices](#best-practices)
- [Fehlerbehebung](#fehlerbehebung)

## Überblick

**NexusBom** und **NexusMRP** sind als komplementäre Systeme konzipiert:

- **NexusBom**: Verwaltet Produktstrukturen, Materialauflösungen und Kostenberechnungen
- **NexusMRP**: Plant Materialbedarfe, plant Produktion und verwaltet Bestände

Zusammen bilden sie eine leistungsstarke Fertigungsplanungslösung.

## Warum integrieren?

| Ohne Integration | Mit Integration |
|------------------|-----------------|
| Manuelle Stücklistenabfragen | Automatische Materialauflösung |
| Statische Planung | Dynamische Bedarfspropagierung |
| Getrennte Systeme | End-to-End-Transparenz |
| Begrenzte Optimierung | Kapazitätsbewusste Planung |

### Hauptvorteile

✅ **Automatische mehrstufige Planung** - MRP nutzt Stückliste zur Auflösung aller Ebenen
✅ **Echtzeit-Kostenanalyse** - Kombination geplanter Aufträge mit Stücklistenkosten
✅ **Änderungsauswirkungsanalyse** - Zeigt, wie Stücklistenänderungen Materialpläne beeinflussen
✅ **Phantom-Teile-Behandlung** - MRP berücksichtigt Phantom-Komponenten der Stückliste
✅ **Alternative Stücklistenunterstützung** - Planung mit verschiedenen Fertigungsrouten

## Architektur

```
┌─────────────────────────────────────────────────────────┐
│                   Ihre Anwendung                         │
└─────────────────────────────────────────────────────────┘
                           │
           ┌───────────────┴───────────────┐
           │                               │
           ▼                               ▼
┌──────────────────────┐       ┌──────────────────────┐
│     NexusBom         │       │     NexusMRP         │
│   (Stücklistenstr.)  │◄──────│   (Planungslogik)    │
└──────────────────────┘       └──────────────────────┘
           │                               │
           │     Materialauflösung          │
           │     Komponentenlisten          │
           │     Kostendaten                │
           └───────────────────────────────┘
```

### Datenfluss

1. **Stücklistendaten laden** → NexusBom erstellt Produktstruktur-Graph
2. **Bedarfe erstellen** → NexusMRP erhält Top-Level-Anforderungen
3. **Stückliste auflösen** → NexusBom liefert Komponentenlisten mit Mengen
4. **MRP berechnen** → NexusMRP propagiert Bedarfe durch Stücklistenebenen
5. **Pläne generieren** → Ausgabe geplanter Aufträge für alle Komponenten

## Integrationsschritte

### Schritt 1: Abhängigkeiten hinzufügen

Fügen Sie beide Bibliotheken zu Ihrer `Cargo.toml` hinzu:

```toml
[dependencies]
# NexusBom - Stücklistenberechnungsmodul
bom-core = { git = "https://github.com/Ricemug/NexusBom" }
bom-calc = { git = "https://github.com/Ricemug/NexusBom" }
bom-graph = { git = "https://github.com/Ricemug/NexusBom" }

# NexusMRP - MRP-Berechnungsmodul
mrp-core = { git = "https://github.com/Ricemug/NexusMRP" }
mrp-calc = { git = "https://github.com/Ricemug/NexusMRP" }
mrp-cache = { git = "https://github.com/Ricemug/NexusMRP" }
```

### Schritt 2: Stücklisten-Graph erstellen

```rust
use bom_core::*;
use bom_graph::BomGraph;

// Definieren Sie Ihre Produktstruktur
let components = vec![
    Component {
        id: ComponentId::new("BIKE-001"),
        description: "Komplettes Fahrrad".to_string(),
        component_type: ComponentType::FinishedProduct,
        standard_cost: Some(Decimal::new(50000, 2)), // $500
        lead_time_days: 5,
        procurement_type: ProcurementType::Make,
    },
    Component {
        id: ComponentId::new("FRAME-001"),
        description: "Fahrradrahmen".to_string(),
        component_type: ComponentType::SubAssembly,
        standard_cost: Some(Decimal::new(20000, 2)), // $200
        lead_time_days: 10,
        procurement_type: ProcurementType::Buy,
    },
    // ... weitere Komponenten
];

let bom_items = vec![
    BomItem {
        parent_id: ComponentId::new("BIKE-001"),
        child_id: ComponentId::new("FRAME-001"),
        quantity: Decimal::ONE,
        sequence: 10,
        is_phantom: false,
    },
    // ... weitere Stücklistenbeziehungen
];

// Erstellen Sie den Stücklisten-Graph
let bom_graph = BomGraph::from_components(&components, &bom_items)?;
```

### Schritt 3: Materialauflösung durchführen

```rust
use bom_calc::ExplosionCalculator;

// Stückliste für bestimmte Menge auflösen
let explosion_calc = ExplosionCalculator::new(&bom_graph);
let explosion_result = explosion_calc.explode(
    &ComponentId::new("BIKE-001"),
    Decimal::from(100), // Menge: 100 Fahrräder
)?;

// Flache Komponentenanforderungen erhalten
let component_requirements = explosion_result.get_flattened_requirements();
```

### Schritt 4: MRP-Bedarfe aus Stückliste erstellen

```rust
use mrp_core::*;
use chrono::NaiveDate;

// Stücklistenauflösung in MRP-Bedarfe umwandeln
let due_date = NaiveDate::from_ymd_opt(2025, 12, 1).unwrap();
let mut demands = Vec::new();

for (component_id, total_qty) in component_requirements {
    let demand = Demand::new(
        component_id.to_string(),
        total_qty,
        due_date,
        DemandType::ProductionOrder,
    );
    demands.push(demand);
}
```

### Schritt 5: MRP mit Stücklisten-Vorlaufzeiten konfigurieren

```rust
use mrp_calc::MRPCalculator;

// MRP-Konfigurationen mit Stücklistendaten erstellen
let mut mrp_configs = Vec::new();

for component in &components {
    let config = MrpConfig {
        item_id: component.id.to_string(),
        lead_time_days: component.lead_time_days,
        procurement_type: component.procurement_type.clone(),
        lot_sizing_rule: LotSizingRule::LotForLot,
        minimum_order_qty: None,
        maximum_order_qty: None,
        order_multiple: None,
        safety_stock: Decimal::ZERO,
    };
    mrp_configs.push(config);
}
```

### Schritt 6: Integrierte MRP-Berechnung ausführen

```rust
// MRP-Rechner initialisieren
let mrp_calculator = MRPCalculator::new(mrp_configs);

// MRP mit stücklistenbasierten Bedarfen ausführen
let mrp_result = mrp_calculator.calculate(
    &demands,
    &existing_supplies,    // Vorhandene Bestellungen oder Produktionsaufträge
    &inventory_balances,   // Aktueller Bestand
)?;

// Geplante Aufträge abrufen
let planned_orders = mrp_result.planned_orders;

println!("{} geplante Aufträge generiert", planned_orders.len());
for order in planned_orders {
    println!("  {} - Menge: {} - Datum: {}",
        order.item_id, order.quantity, order.due_date);
}
```

## Codebeispiele

### Vollständiges Integrationsbeispiel

```rust
use bom_core::*;
use bom_graph::BomGraph;
use bom_calc::ExplosionCalculator;
use mrp_core::*;
use mrp_calc::MRPCalculator;
use rust_decimal::Decimal;
use chrono::NaiveDate;

fn integrated_planning_example() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Stücklistenstruktur erstellen
    let bom_graph = build_bicycle_bom()?;

    // 2. Kundenauftrag empfangen
    let customer_order = CustomerOrder {
        product_id: "BIKE-001".to_string(),
        quantity: Decimal::from(100),
        due_date: NaiveDate::from_ymd_opt(2025, 12, 1).unwrap(),
    };

    // 3. Stückliste auflösen für Komponentenanforderungen
    let explosion_calc = ExplosionCalculator::new(&bom_graph);
    let explosion = explosion_calc.explode_with_lead_time_offset(
        &ComponentId::new(&customer_order.product_id),
        customer_order.quantity,
        customer_order.due_date,
    )?;

    // 4. In MRP-Bedarfe umwandeln
    let demands: Vec<Demand> = explosion
        .items
        .iter()
        .map(|item| Demand {
            item_id: item.component_id.to_string(),
            quantity: item.total_quantity,
            due_date: item.required_date,
            demand_type: DemandType::DependentDemand,
            source_id: Some(customer_order.product_id.clone()),
        })
        .collect();

    // 5. MRP-Berechnung ausführen
    let mrp_configs = extract_mrp_configs_from_bom(&bom_graph);
    let calculator = MRPCalculator::new(mrp_configs);

    let mrp_result = calculator.calculate(
        &demands,
        &vec![], // Keine vorhandenen Lieferungen
        &vec![], // Kein vorhandener Bestand
    )?;

    // 6. Geplante Aufträge ausgeben
    println!("Geplante Aufträge für Kundenauftrag {}:", customer_order.product_id);
    for order in mrp_result.planned_orders {
        println!("  Auftrag: {} - Menge: {} - Start: {} - Fällig: {}",
            order.item_id,
            order.quantity,
            order.order_date,
            order.due_date
        );
    }

    // 7. Gesamtkosten mit Stückliste berechnen
    let total_cost = calculate_order_cost(&bom_graph, &mrp_result.planned_orders)?;
    println!("Gesamtmaterialkosten: ${:.2}", total_cost);

    Ok(())
}

fn extract_mrp_configs_from_bom(bom_graph: &BomGraph) -> Vec<MrpConfig> {
    bom_graph
        .get_all_components()
        .iter()
        .map(|component| MrpConfig {
            item_id: component.id.to_string(),
            lead_time_days: component.lead_time_days,
            procurement_type: component.procurement_type.clone(),
            lot_sizing_rule: LotSizingRule::LotForLot,
            minimum_order_qty: None,
            maximum_order_qty: None,
            order_multiple: None,
            safety_stock: Decimal::ZERO,
        })
        .collect()
}
```

### Phantom-Komponenten behandeln

```rust
// Phantom-Teile werden sofort verbraucht, nicht separat geplant
fn handle_phantom_components(
    bom_graph: &BomGraph,
    explosion: &ExplosionResult,
) -> Vec<Demand> {
    explosion
        .items
        .iter()
        .filter(|item| {
            // Phantom-Komponenten in MRP-Planung überspringen
            let component = bom_graph.get_component(&item.component_id).unwrap();
            !matches!(component.component_type, ComponentType::Phantom)
        })
        .map(|item| Demand {
            item_id: item.component_id.to_string(),
            quantity: item.total_quantity,
            due_date: item.required_date,
            demand_type: DemandType::DependentDemand,
            source_id: None,
        })
        .collect()
}
```

### Inkrementelle Aktualisierungen

```rust
use mrp_cache::IncrementalCache;

// Caching für effiziente Neuplanung nutzen
fn incremental_replanning(
    bom_graph: &BomGraph,
    mrp_cache: &mut IncrementalCache,
    changed_demands: Vec<Demand>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Nur betroffene Artikel neu berechnen
    let affected_items = mrp_cache.get_affected_items(&changed_demands);

    // Nur geänderte Top-Level-Artikel neu auflösen
    let explosion_calc = ExplosionCalculator::new(&bom_graph);
    for demand in changed_demands {
        let explosion = explosion_calc.explode(
            &ComponentId::new(&demand.item_id),
            demand.quantity,
        )?;

        // Cache mit neuen Auflösungen aktualisieren
        mrp_cache.update_explosion(&demand.item_id, explosion);
    }

    // MRP nur für betroffene Artikel neu berechnen
    let mrp_result = mrp_cache.calculate_incremental(&affected_items)?;

    Ok(())
}
```

## Best Practices

### 1. Stücklistenauflösungen cachen

```rust
use std::collections::HashMap;

struct BomCache {
    explosions: HashMap<String, ExplosionResult>,
}

impl BomCache {
    fn get_or_calculate(&mut self,
        bom_graph: &BomGraph,
        item_id: &str,
        quantity: Decimal
    ) -> &ExplosionResult {
        self.explosions.entry(item_id.to_string()).or_insert_with(|| {
            ExplosionCalculator::new(bom_graph)
                .explode(&ComponentId::new(item_id), quantity)
                .unwrap()
        })
    }
}
```

### 2. Datenkonsistenz validieren

```rust
fn validate_bom_mrp_consistency(
    bom_graph: &BomGraph,
    mrp_configs: &[MrpConfig],
) -> Result<(), String> {
    // Sicherstellen, dass alle Stücklistenkomponenten MRP-Konfigurationen haben
    for component in bom_graph.get_all_components() {
        let has_config = mrp_configs
            .iter()
            .any(|cfg| cfg.item_id == component.id.to_string());

        if !has_config {
            return Err(format!(
                "Komponente {} in Stückliste hat keine MRP-Konfiguration",
                component.id
            ));
        }
    }

    Ok(())
}
```

### 3. Vorlaufzeit-Offsets handhaben

```rust
// Auftragsdaten unter Berücksichtigung der Stücklistenebenen berechnen
fn calculate_order_dates_with_bom_levels(
    bom_graph: &BomGraph,
    top_level_due_date: NaiveDate,
) -> HashMap<String, NaiveDate> {
    let mut order_dates = HashMap::new();

    // Stückliste rückwärts durchlaufen (bottom-up)
    for level in bom_graph.get_levels_bottom_up() {
        for component in level {
            let lead_time = component.lead_time_days;
            let parent_dates: Vec<NaiveDate> = bom_graph
                .get_parents(&component.id)
                .iter()
                .map(|parent_id| {
                    *order_dates.get(&parent_id.to_string())
                        .unwrap_or(&top_level_due_date)
                })
                .collect();

            let earliest_parent_date = parent_dates.iter().min()
                .unwrap_or(&top_level_due_date);

            let order_date = *earliest_parent_date - chrono::Duration::days(lead_time as i64);
            order_dates.insert(component.id.to_string(), order_date);
        }
    }

    order_dates
}
```

### 4. Performance überwachen

```rust
use std::time::Instant;

fn benchmark_integrated_system() {
    let start = Instant::now();

    // Stücklistenauflösung
    let explosion_start = Instant::now();
    let explosion = explode_bom();
    println!("Stücklistenauflösung: {:?}", explosion_start.elapsed());

    // MRP-Berechnung
    let mrp_start = Instant::now();
    let mrp_result = calculate_mrp();
    println!("MRP-Berechnung: {:?}", mrp_start.elapsed());

    println!("Gesamtzeit: {:?}", start.elapsed());
}
```

## Fehlerbehebung

### Problem: Zirkuläre Stücklistenabhängigkeiten

**Problem**: MRP-Berechnung schlägt aufgrund zirkulärer Referenzen in Stückliste fehl

**Lösung**:
```rust
// Stücklisten-Graph-Validierung verwenden
if let Err(e) = bom_graph.validate_no_cycles() {
    eprintln!("Stückliste enthält zirkuläre Abhängigkeiten: {}", e);
    // Fehler angemessen behandeln
}
```

### Problem: Nicht übereinstimmende Vorlaufzeiten

**Problem**: MRP-Aufträge werden zu spät berechnet

**Lösung**:
```rust
// Vorlaufzeiten immer von Stückliste zu MRP-Konfigurationen synchronisieren
for component in bom_graph.get_all_components() {
    let mrp_config = mrp_configs.iter_mut()
        .find(|cfg| cfg.item_id == component.id.to_string())
        .unwrap();

    mrp_config.lead_time_days = component.lead_time_days;
}
```

### Problem: Speichernutzung bei großen Stücklisten

**Problem**: Hoher Speicherverbrauch bei komplexen Produktstrukturen

**Lösung**:
```rust
// Streaming-Auflösung statt vollständiger Materialisierung verwenden
let explosion_stream = ExplosionCalculator::new(&bom_graph)
    .explode_streaming(&root_id, quantity);

for batch in explosion_stream.chunks(1000) {
    process_demands_batch(batch);
}
```

## Verwandte Dokumentation

- [NexusBom Dokumentation](https://github.com/Ricemug/NexusBom)
- [NexusMRP Dokumentation](../README.md)
- [Dynamische Zeitbuckets](./DYNAMIC_TIME_BUCKETS.md)
- [Negative Bestandsbehandlung](./NEGATIVE_INVENTORY.md)

## Support

Bei Integrationsfragen:
- Erstellen Sie ein Issue auf [NexusMRP GitHub](https://github.com/Ricemug/NexusMRP/issues)
- E-Mail: xiaoivan1@proton.me

---

**Viel Erfolg bei der Planung! 🚀**
