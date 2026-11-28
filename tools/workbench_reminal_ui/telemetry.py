"""Read-only interpretation of the receiver's published schema and observations."""
from __future__ import annotations


def schema_key(group):
    return ('attack' if 1 <= group <= 8 else
            {0: 'object', 9: 'input', 10: 'policy'}.get(group, 'feature' if group >= 11 else ''))


def observation_rows(schema, data, elapsed_ms, available, *, object_id='*', group='*', reference=False, available_only=False):
    """Project every published schema field without turning missing evidence into zero."""
    records = {(str(o['object_id']), o['group']): o for o in data.get('objects', [])}
    if reference:
        groups = [0, *range(1, 9), 9, 10, *range(schema.get('feature_group_start', 11),
                                                    schema.get('feature_group_start', 11) + len(schema.get('features', [])))]
        selections = [(object_id, g, None) for g in groups if group == '*' or str(g) == group]
    else:
        selections = [(oid, g, record) for (oid, g), record in records.items()
                      if (object_id == '*' or object_id == oid) and (group == '*' or str(g) == group)]
    rows = []
    for oid, g, record in selections:
        fields = {f['index']: f for f in schema.get(schema_key(g), [])}
        values = record.get('values', []) if record else []
        indices = sorted(set(fields) | set(range(len(values))))
        age = record.get('age_ms', 0) + elapsed_ms if record else None
        expired = age is not None and age >= 1000
        stale = bool(record and (record.get('stale', False) or age >= 250 or not available))
        for index in indices:
            field = fields.get(index)
            value = values[index] if index < len(values) else None
            if not field and value is None:
                continue
            field = field or {'index': index, 'label': f'Field {index}', 'name': f'field_{index}'}
            if reference or expired:
                value = None
            if available_only and not reference and value is None:
                continue
            rows.append({'key': f'{oid}|{g}|{index}', 'object': oid, 'group': g, 'field': field,
                         'value': value, 'age': age, 'state': 'schema' if reference else 'unavailable' if value is None else 'stale' if stale else 'live',
                         'record': record})
    return rows


def assistance_summary(schema, data, object_id, elapsed_ms, available):
    records = {o['group']: o for o in data.get('objects', []) if o['object_id'] == object_id}

    def fresh(record):
        return bool(available and record and not record['stale'] and record['age_ms'] + elapsed_ms < 250)

    def value(record, index, key='display'):
        values = record.get('values', []) if record else []
        item = values[index] if index < len(values) else None
        return item.get(key) if item else None

    native, policy = records.get(0), records.get(10)
    valid = fresh(policy) and value(policy, 0) == 'true'

    def bits(index):
        raw = value(policy, index, 'raw') if valid else None
        return int(raw, 16) if raw is not None else None

    present, selected, safety = bits(2), bits(1), bits(5)
    lines = [f'Selected object: {object_id or "None"}',
             'Delivery limit: proposal selection and final mapper output do not confirm game execution.']
    for label, index in [('Observed action', 0), ('Previous action', 1), ('Damage (%)', 19),
                         ('Animation frame', 4), ('Victim hitlag', 17), ('Hitlag remaining', 18), ('Shieldstun', 37)]:
        observed = value(native, index) if fresh(native) else None
        lines.append(f'{label}: {observed if observed is not None else "Unknown / unavailable"}')
    mapped = records.get(9)
    lines.append('Final mapped buttons: ' + str(value(mapped, 11) if fresh(mapped) else 'Unknown / stale'))
    lines.append('Final mapped left stick: ' + (', '.join(str(value(mapped, i)) for i in (12, 13)) if fresh(mapped) else 'Unknown / stale'))
    flags = []
    for group, record in records.items():
        if fresh(record):
            for field in schema.get(schema_key(group), []):
                if field.get('kind') == 'Boolean' and value(record, field['index']) == 'true':
                    flags.append(f"{field.get('label') or field.get('name', 'Field')} · group {group}")
    lines.append('Fresh active flags: ' + ('; '.join(flags) or 'None observed; missing/stale is not false'))
    rows = []
    for index, feature in enumerate(schema.get('features', [])):
        detail = records.get(schema.get('feature_group_start', 11) + index)
        configured = value(detail, 0) if fresh(detail) else value(policy, feature['configured_index']) if fresh(policy) else None
        enabled = 'Enabled preference' if configured == 'true' else 'Disabled preference' if configured == 'false' else 'Unknown'
        mask = safety if feature.get('safety') is not None else present
        bit = feature.get('safety') if feature.get('safety') is not None else feature['owner']
        proposed = bool(mask & int(bit)) if mask is not None else None
        winner = bool(selected & int(feature['owner'])) if selected is not None else None
        evidence = ('Unknown / stale' if proposed is None else 'No current proposal' if not proposed else
                    'Selected proposal (not delivery)' if winner else 'Proposed; not selected')
        constraints = []
        if not fresh(detail):
            constraints.append('Details unknown / stale')
        else:
            hold, raw = value(detail, 1), value(detail, 2, 'raw')
            if hold is not None:
                constraints.append('Hold trigger pressed' if hold == 'true' else 'Hold trigger released')
            if raw is None:
                constraints.append('Constraint evidence unavailable')
            else:
                flags_value = int(raw, 16)
                labels = [(1, 'Policy scope unavailable'), (2, 'Animation stopped'), (4, 'Native readiness not confirmed'),
                          (8, 'Different input owner selected'), (16, 'Required hold not pressed')]
                constraints.extend(label for bit_value, label in labels if flags_value & bit_value)
                if not constraints:
                    constraints.append('No reported constraints (not proof eligible)')
        rows.append((feature['label'], enabled, evidence,
                     str(detail.get('mode') or 'Mode unavailable') if fresh(detail) else 'Mode unknown / stale',
                     str(detail.get('recipe') or 'Recipe unavailable') if fresh(detail) else 'Recipe unknown / stale',
                     ' · '.join(constraints), feature['description']))
    return '\n'.join(lines), rows
