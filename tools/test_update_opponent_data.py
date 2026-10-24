import unittest
from tools import update_opponent_data as data

class ImportTests(unittest.TestCase):
    def test_multihit_gaps_and_unknown_expressions_are_not_filled_in(self):
        self.assertEqual(data.spans('11/14/18—19'),[[11,11],[14,14],[18,19]])
        self.assertEqual(data.spans('3—4(5—11/12—25)'),[[3,4],[5,11],[12,25]])
        self.assertEqual(data.spans('5/7/9/11...'),[])
        self.assertEqual(data.spans('5—**'),[])
        self.assertIsNone(data.integer('40/52'))
    def test_conditional_cancel_notes_never_become_normal_whiff_rules(self):
        def move(note):
            return '<div class="movecontainer"><div class="movename">Down Tilt</div><div class="totalframes">20</div><div class="activeframes">4—6</div><div class="notes">'+note+'</div></div>'
        parsed=data.moves(move('Can cancel on hit')+move('--')*9)
        self.assertFalse(parsed[0]['whiff_eligible']); self.assertTrue(parsed[1]['whiff_eligible'])
        self.assertEqual(parsed[1]['total'],20)
    def test_optional_autocancel_and_landing_hit_facts_survive_import(self):
        source='<div class="movecontainer"><div class="movename">Down Air</div><div class="notes">Landing hit on frame 1. Autocancels on frame 1-17 and 48 onward</div></div>'
        parsed=data.moves(source*10)[0]
        self.assertEqual(parsed['raw']['landing_hit_frame'],'1')
        self.assertEqual(parsed['raw']['autocancel'],'1-17 and 48 onward')
        self.assertFalse(parsed['whiff_eligible'])
if __name__ == '__main__': unittest.main()
