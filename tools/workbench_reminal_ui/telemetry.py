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
        age = record['age_ms'] + elapsed_ms if record and record.get('age_ms') is not None else None
        expired = record is not None and (age is None or age >= 1000)
        stale = bool(record and (record.get('stale', True) or age is None or age >= 250 or not available))
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
        return bool(available and record and not record.get('stale', True)
                    and record.get('age_ms') is not None and record['age_ms'] + elapsed_ms < 250)

    def value(record, index, key='display'):
        values = record.get('values', []) if record else []
        item = values[index] if index < len(values) else None
        return item.get(key) if item else None

    native, policy = records.get(0), records.get(10)
    valid = fresh(policy) and value(policy, 0) == 'true'

    def bits(index):
        raw = value(policy, index, 'raw') if valid else None
        try:
            return int(raw, 16) if raw is not None else None
        except (ValueError, TypeError):
            return None

    present, selected, safety = bits(2), bits(1), bits(5)
    lines = [f'Selected object: {object_id or "None"}',
             'Delivery limit: proposal selection and final mapper output do not confirm game execution.']
    for label, index in [('Observed action', 0), ('Previous action', 1), ('Damage (%)', 19),
                         ('Animation frame', 4), ('Victim hitlag', 17), ('Hitlag remaining', 18), ('Shieldstun', 37)]:
        observed = value(native, index) if fresh(native) else None
        lines.append(f'{label}: {observed if observed is not None else "Unknown / unavailable"}')
    for label, index in [('Policy scope valid', 0), ('Native ready to act', 6), ('Policy animation stopped', 10)]:
        observed = value(policy, index) if fresh(policy) and (index == 0 or valid) else None
        lines.append(f'{label}: {observed if observed is not None else "Unknown / stale"}')
    for label, index in [('Committed input-owner mask (not delivery)', 3), ('Requested button mask (not delivery)', 4),
                         ('Policy native frame', 11), ('Proposed stick X (quantized; not delivery)', 12),
                         ('Proposed stick Y (quantized; not delivery)', 13)]:
        observed = value(policy, index) if valid else None
        lines.append(f'{label}: {observed if observed is not None else "Unknown / stale"}')
    mapped = records.get(9)
    def mapped_value(index):
        observed = value(mapped, index) if fresh(mapped) else None
        return str(observed) if observed is not None else 'Unknown / unavailable'
    lines.append('Final mapped buttons: ' + mapped_value(11))
    lines.append('Final mapped left stick: ' + ', '.join(mapped_value(i) for i in (12, 13)))
    flags = []
    for group, record in records.items():
        if fresh(record):
            for field in schema.get(schema_key(group), []):
                preference = (group == 10 and field['index'] >= 16) or (group >= 11 and field['index'] == 0)
                if not preference and field.get('kind') == 'Boolean' and value(record, field['index']) == 'true':
                    flags.append(f"{field.get('label') or field.get('name', 'Field')} · group {group}")
    lines.append('Fresh observed true flags (not tool activity): ' + ('; '.join(flags) or 'None observed; missing/stale is not false'))
    rows = []
    for index, feature in enumerate(schema.get('features', [])):
        detail = records.get(schema.get('feature_group_start', 11) + index)
        configured = value(detail, 0) if fresh(detail) else None
        if configured is None and fresh(policy):
            configured = value(policy, feature['configured_index'])
        enabled = 'Enabled preference' if configured == 'true' else 'Disabled preference' if configured == 'false' else 'Unknown'
        mask = safety if feature.get('safety') is not None else present
        bit = feature.get('safety') if feature.get('safety') is not None else feature['owner']
        proposed = bool(mask & int(bit)) if mask is not None else None
        winner = bool(selected & int(feature['owner'])) if selected is not None else None
        explicit_proposed = value(detail, 3) if fresh(detail) else None
        explicit_selected = value(detail, 4) if fresh(detail) else None
        shared_owner = sum(f['owner'] == feature['owner'] for f in schema.get('features', [])) > 1
        if explicit_proposed in {'true', 'false'}:
            proposed = explicit_proposed == 'true'
        elif proposed and shared_owner and feature.get('safety') is None:
            proposed = None
        if explicit_selected in {'true', 'false'}:
            winner = explicit_selected == 'true'
        elif shared_owner:
            winner = None
        evidence = ('Unknown / stale' if proposed is None else 'No current proposal' if not proposed else
                    'Proposed; selection unknown' if winner is None else
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
                try:
                    flags_value = int(raw, 16)
                except (ValueError, TypeError):
                    constraints.append('Constraint evidence unavailable')
                else:
                    labels = [(1, 'Policy scope unavailable'), (8, 'Different input owner selected'),
                              (16, 'Required hold not pressed')]
                    reported = [label for bit_value, label in labels if flags_value & bit_value]
                    if flags_value & ~25:
                        reported.append(f'Unknown constraint bits: {flags_value & ~25:#x}')
                    constraints.extend(reported or ['No reported constraints (not proof eligible)'])
        rows.append((feature['label'], enabled, evidence,
                     str(detail.get('mode') or 'Mode unavailable') if fresh(detail) else 'Mode unknown / stale',
                     str(detail.get('recipe') or 'Recipe unavailable') if fresh(detail) else 'Recipe unknown / stale',
                     ' · '.join(constraints), feature['description']))
    return '\n'.join(lines), rows
