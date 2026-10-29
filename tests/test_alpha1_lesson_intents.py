import re
import json
import importlib.util
import tomllib
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1] / 'solutions/seamgrim_ui_mvp'
PIN = re.compile(r'^\s*([^\s:]+):\s*수\s*(?:<-|:=|=)\s*\(([^)]+)\)\s*매김\s*\{\s*범위:\s*([^;]+?)\.\s*간격:\s*([^;]+?)\.\s*\}', re.M)


class LessonIntentPreservation(unittest.TestCase):
    def test_every_rewritten_lesson_preserves_metadata_and_controls(self):
        manifest = json.loads((ROOT / 'lessons_v25_alpha1/dispositions.detjson').read_text('utf-8'))
        folders = sorted(ROOT / row['source'].removesuffix('/lesson.ddn') for row in manifest['lessons']
                         if row.get('source', '').startswith('lessons_v25_alpha1/'))
        self.assertEqual(len(folders), 155)  # Preserved registered corpus; unfinished candidates excluded.
        for folder in folders:
            with self.subTest(lesson=folder.name):
                origins = [ROOT / category / folder.name for category in
                           ('lessons', 'lessons_rewrite_v1', 'seed_lessons_v1')]
                before = next(path for path in origins if path.is_dir())
                new_meta = tomllib.loads((folder / 'meta.toml').read_text('utf-8'))
                if not (before / 'meta.toml').is_file():
                    original = json.loads((before / 'lesson.json').read_text('utf-8'))
                    for key in ('title', 'description', 'grade', 'subject', 'keywords'):
                        self.assertEqual(new_meta[key], original[key], key)
                    original_text = (before / 'text.md').read_text('utf-8')
                    for goal in new_meta['goals']:
                        self.assertIn(goal, original_text)
                    for mission in new_meta.get('missions', []):
                        # Older derived metadata removes Markdown inline-code
                        # delimiters; compare the unchanged words, not styling.
                        self.assertIn(mission.replace('`', ''), original_text.replace('`', ''))
                    self.assertEqual(new_meta['required_views'], ['graph'])
                else:
                    old_meta = tomllib.loads((before / 'meta.toml').read_text('utf-8'))
                    for key in ('title', 'description', 'goals', 'missions', 'required_views', 'first_delight_controls'):
                        self.assertEqual(new_meta.get(key), old_meta.get(key), key)
                old_source = (before / 'lesson.ddn').read_text('utf-8')
                new_source = (folder / 'lesson.ddn').read_text('utf-8')
                old_pins = {row[0]: tuple(re.sub(r'\s+', '', value) for value in row[1:]) for row in PIN.findall(old_source)}
                new_pins = {row[0]: tuple(re.sub(r'\s+', '', value) for value in row[1:]) for row in PIN.findall(new_source)}
                if folder.name in ('econ_supply_demand', 'high_econ_elasticity', 'high_econ_interest', 'college_econ_growth', 'college_econ_productivity', 'college_econ_utility', 'college_physics_orbit', 'college_physics_thermal', 'high_physics_damped', 'high_physics_energy', 'high_physics_wave', 'physics_projectile', 'ssot_edu_phys_p005_03_specific_heat_TQ', 'ssot_edu_phys_p005_04_heating_curve_TQ_piecewise', 'ssot_edu_phys_p005_05_newton_cooling_Tt_discrete', 'ssot_edu_phys_p006_01_inverse_square_intensity', 'ssot_edu_phys_p006_02_thin_lens_di_vs_do', 'ssot_edu_phys_p006_03_magnification_vs_do', 'ssot_edu_phys_p006_04_photoelectric_ke_vs_frequency', 'ssot_edu_phys_p006_05_double_slit_fringe_spacing_vs_d', 'ssot_edu_phys_p015_01_pendulum_L_T2', 'ssot_edu_phys_p015_02_spring_m_T2', 'ssot_edu_phys_p015_03_centripetal_v_a'):
                    self.assertEqual(old_pins, {}, 'these original lessons have no controls')
                elif folder.name in ('econ_supply_demand_seed_v1', 'physics_heat_diffusion_seed_v1',
                    'ssot_edu_phys_p004_02_voltage_divider_Vin_Vout', 'ssot_edu_phys_p004_03_parallel_resistors_IV',
                    'ssot_edu_phys_p013_03_adiabatic_pv_v_p', 'ssot_edu_phys_p018_03_debroglie_lambda_of_p',
                    'ssot_edu_phys_p018_04_wien_lambda_max_of_T'):
                    self.assertEqual(old_pins, {})
                    self.assertNotIn('매김', old_source)
                elif folder.name not in ('rep_physics_velocity_history_v1', 'physics_motion'):
                    if folder.name in ('ssot_edu_phys_p012_04_doppler_vs_source_speed', 'ssot_edu_phys_p013_05_carnot_Th_eta', 'ssot_edu_phys_p015_04_dynamic_pressure_v_q', 'ssot_edu_phys_p018_05_hydrogen_energy_levels'):
                        self.assertEqual(old_pins, {}, 'these original lessons have no controls')
                    else:
                        if folder.name in ('college_math_series', 'high_math_polynomial', 'ssot_edu_phys_p012_01_standing_wave_yx_snapshot', 'ssot_edu_phys_p012_02_standing_wave_intensity_Ix', 'ssot_edu_phys_p012_03_beats_yt'):
                            self.assertEqual(old_pins, {}, 'these original lessons have no controls')
                        elif folder.name in ('ssot_edu_phys_p013_01_isothermal_pv_v_p', 'ssot_edu_phys_p013_02_isothermal_work_v_W', 'ssot_edu_phys_p017_01_critical_angle_deg_of_ratio', 'ssot_edu_phys_p017_05_brewster_angle_deg_of_ratio', 'ssot_edu_phys_p012_05_fourier_square_wave_yx', 'ssot_edu_phys_p011_04_resonance_amplitude_vs_omega', 'ssot_edu_phys_p015_05_linear_drag_fall_t_v'):
                            self.assertEqual(old_pins, {}, 'these original lessons have no controls')
                        else:
                            if folder.name == 'math_parabola':
                                self.assertEqual(old_pins, {}, 'canonical parabola has no controls')
                            else:
                                self.assertTrue(old_pins)
                self.assertEqual(new_pins, old_pins, 'all default/range/step values must remain')
                self.assertNotIn('보임 {', new_source)
                self.assertNotIn('인것 일때', new_source)

    def test_17_representatives_and_six_candidates_have_explicit_dispositions(self):
        manifest = json.loads((ROOT / 'lessons_v25_alpha1/dispositions.detjson').read_text('utf-8'))
        rows = manifest['lessons']
        self.assertEqual(len({row['id'] for row in rows}), 162)
        self.assertEqual(sum(row['cohort'] == '대표17' for row in rows), 17)
        self.assertEqual(sum(row['disposition'] == '이월' for row in rows), 4)
        self.assertEqual(sum(row['disposition'] == '알파1 새 구현' for row in rows), 158)
        for row in rows:
            if row['disposition'] == '이월':
                self.assertEqual(row['stage'], '알파2')
                self.assertTrue(row['reason'])
                self.assertNotIn('source', row)
            else:
                self.assertTrue((ROOT / row['source']).is_file())

    def test_server_catalog_uses_only_current_runnable_sources(self):
        spec = importlib.util.spec_from_file_location('alpha_lesson_server', ROOT / 'tools/ddn_exec_server.py')
        server = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(server)
        payload = server.build_lesson_inventory_payload('reps_only')
        self.assertEqual(payload['count'], 3)  # Four menus: three files plus inline straight-line example.
        self.assertEqual(len(payload['deferred']), 0)  # Curriculum stays in custody, not alpha.1 UI.
        manifest = json.loads((ROOT / 'lessons_v25_alpha1/dispositions.detjson').read_text('utf-8'))
        allowed = {'rep_phys_projectile_xy_v1', 'rep_phys_pendulum_wave1_v1', 'rep_grid_game_state_drop_v1'}
        expected = {row['id']: row['source'] for row in manifest['lessons'] if row['id'] in allowed}
        self.assertEqual({row['id']: row['ddn_path'][0] for row in payload['lessons']}, expected)
        self.assertEqual(len(server.build_public_lesson_index_payload('reps_only')['lessons']), 3)


if __name__ == '__main__':
    unittest.main()
