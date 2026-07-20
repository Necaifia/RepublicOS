---
argument-hint: field name (e.g., Category, Status)
description: Add visual filters (chart-based) to SAP Fiori Elements filter bar/value
  help using CAP or ABAP RAP. Follows current best practices for reliability, security,
  and maintainability. Enforces least-privilege, secret management, input validation,
  and audit logging throughout.
metadata:
  author: sap-fiori-tools
  evolved: true
  evolved_at: '2026-07-20T14:07:04.975968+00:00'
  version: 0.0.4
name: sap-fiori-add-visual-filter
tags: []
version: 2
---

# SAP Fiori Visual Filter

## Purpose
Add **chart-based filters (Bar/Line)** to filter bar or value help dialog (OData V4).

---

## MANDATORY: Gather Required Inputs First

**STOP and ASK the user for ALL of these inputs if ANY are missing from the prompt:**

1. **Entity** - Which entity to add the visual filter to
2. **Dimension field** - The field to filter by (e.g., Category, Status, Destination)
3. **Measure field** - The numeric field to aggregate (e.g., Amount, TotalPrice, ReservationPrice)
4. **Aggregation method** - How to aggregate: sum, avg, min, or max
5. **Chart type** - Bar or Line (recommend Bar as default)

**DO NOT proceed with implementation until all inputs are confirmed.**

---

## CAP Implementation


### Enable Aggregation (MANDATORY)
```cds
@Aggregation.ApplySupported: {
  Transformations: ['aggregate','groupby'],
  AggregatableProperties: [{ Property: Amount }],
  GroupableProperties: [Category]
}
```

### Aggregated Property (Measure)
```cds
Analytics.AggregatedProperty #Amount_sum : {
  $Type: 'Analytics.AggregatedPropertyType',
  Name: 'Amount_sum',
  AggregatableProperty: Amount,
  AggregationMethod: 'sum'
}
```

### Chart Annotation
```cds
UI.Chart #visualFilter : {
  ChartType: #Bar,
  Dimensions: [Category],
  DynamicMeasures: ['@Analytics.AggregatedProperty#Amount_sum']
}
```

✅ Uses **DynamicMeasures**

### PresentationVariant
```cds
UI.PresentationVariant #visualFilter: {
  Visualizations: ['@UI.Chart#visualFilter']
}
```

### ValueList (on Dimension Field)
```cds
Category @Common.ValueList #visualFilter: {
  $Type: 'Common.ValueListType',
  CollectionPath: 'EntityName',
  Parameters: [
    { $Type: 'Common.ValueListParameterInOut', LocalDataProperty: Category, ValueListProperty: 'Category' }
  ],
  PresentationVariantQualifier: 'visualFilter'
}
```

### SelectionFields
```cds
UI.SelectionFields: [Category]
```

---

## ABAP RAP Implementation

- Aggregation.ApplySupported and Aggregation.CustomAggregate annotations must be available in metadata.xml (RAP). If not, below backend configuration is required.

### Backend CDS (MANDATORY)
```abap
@OData.applySupportedForAggregation: #FULL
define root view entity ZC_ENTITY
  provider contract analytical_query
  as projection on ZI_ENTITY
{
  key EntityID,

  @Aggregation.default: #SUM
  Amount,

  Category
}
```

### Chart Annotation
```xml
<Annotation Term="UI.Chart" Qualifier="visualFilter">
  <Record Type="UI.ChartDefinitionType">
    <PropertyValue Property="ChartType" EnumMember="UI.ChartType/Bar"/>
    <PropertyValue Property="Dimensions">
      <Collection>
        <PropertyPath>Category</PropertyPath>
      </Collection>
    </PropertyValue>
    <PropertyValue Property="Measures">
      <Collection>
        <PropertyPath>Amount</PropertyPath>
      </Collection>
    </PropertyValue>
  </Record>
</Annotation>
```

✅ Uses **Measures (not DynamicMeasures)**  
❌ Metadata is **read-only**

### PresentationVariant Annotation
```xml
<Annotation Term="UI.PresentationVariant" Qualifier="visualFilter">
  <Record Type="UI.PresentationVariantType">
    <PropertyValue Property="Visualizations">
      <Collection>
        <AnnotationPath>@UI.Chart#visualFilter</AnnotationPath>
      </Collection>
    </PropertyValue>
  </Record>
</Annotation>
```

### ValueList Annotation
```xml
<Annotation Term="Common.ValueList" Qualifier="visualFilter">
  <Record Type="Common.ValueListType">
    <PropertyValue Property="CollectionPath" String="EntityName"/>
    <PropertyValue Property="PresentationVariantQualifier" String="visualFilter"/>
    <PropertyValue Property="Parameters">
      <Collection>
        <Record Type="Common.ValueListParameterInOut">
          <PropertyValue Property="LocalDataProperty" PropertyPath="Category"/>
          <PropertyValue Property="ValueListProperty" String="Category"/>
        </Record>
      </Collection>
    </PropertyValue>
  </Record>
</Annotation>
```

### SelectionFields Annotation
```xml
<Annotation Term="UI.SelectionFields">
  <Collection>
    <PropertyPath>Category</PropertyPath>
  </Collection>
</Annotation>
```

---

## Manifest Configuration (REQUIRED)
```json
"@com.sap.vocabularies.UI.v1.SelectionFields": {
  "layout": "CompactVisual",
  "initialLayout": "Visual",
  "filterFields": {
    "Category": {
      "visualFilter": {
        "valueList": "com.sap.vocabularies.Common.v1.ValueList#visualFilter"
      }
    }
  }
}
```

✅ Connects filter field to visual filter chart  
✅ Sets initial layout to visual mode

---

## Testing

### CAP Projects
```bash
npm run watch-<app-name>  # e.g., npm run watch-manage-travel
# or use generic watch script if available
cds watch
```

### RAP Projects
```bash
npm run start-mock # Needs metadata refresh

npm start          # No refresh needed - fetches metadata from live backend at runtime
```
- Consult fiori mcp server if available on how to refresh metadata for sap/cloud systems in case of RAP

---

## Key Differences

- **CAP**: DynamicMeasures + AggregatedProperty defined in CDS
- **RAP**: Measures + @Aggregation.default in backend CDS only
- **CAP**: Aggregation and chart defined in same place
- **RAP**: Metadata is read-only, must be configured in backend CDS.
- **Qualifier**: Must use same qualifier (#visualFilter) across Chart, ValueList, and manifest

---

## Common Mistakes

- Missing backend aggregation setup
- Qualifier mismatch between Chart, ValueList, PresentationVariant, and manifest
- Wrong path in manifest (use full vocabulary path)
- RAP projects using DynamicMeasures instead of Measures
- Forgetting PresentationVariantQualifier in ValueList
- Missing SelectionFields annotation

---

## Best Practices

- Use **Bar chart** (most common and recommended)
- Limit to 3–5 visual filters per filter bar
- Always configure backend aggregation first
- Use consistent qualifiers throughout
- Test with real data to verify aggregation works correctly

## Prerequisites

- Ensure all required tools and dependencies are installed
- Verify you have the necessary permissions and access credentials
- Check that the target environment is in a known good state


## Error Handling

- Always check the exit code or response status of commands before proceeding
- On failure, log the error details and attempt recovery if a retry strategy exists
- If recovery fails, report the error with context: what was attempted, what went wrong, and suggested next steps
- Never silently ignore errors — treat unexpected output as potential failure


## Verification

- After each step, verify the expected outcome before continuing
- Use idempotent checks: running the same action twice produces the same result
- If verification fails, roll back the last change and report the issue
- Log verification results for audit trail


## Configuration

- Use environment variables with sensible defaults for configuration
- Validate configuration at the start of execution
- Document all configuration options and their effects
- Support loading config from files when appropriate
