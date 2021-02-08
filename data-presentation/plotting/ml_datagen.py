sql = """SELECT
    Pipeline.status,
    duration,
    ref,
    jobs,
    COUNT(AccessLog.id) AS accessCount
FROM Pipeline
LEFT OUTER JOIN AccessLog
    ON Pipeline.id = AccessLog.pipeline
WHERE
    Pipeline.status IS NOT NULL
    AND
        jobs != ""
    AND
        (AccessLog.isAutomatic == 0 OR AccessLog.isAutomatic IS NULL)
    AND
        (AccessLog.isIrrelevant == 0 OR AccessLog.isIrrelevant IS NULL)
GROUP BY Pipeline.id
ORDER BY accessCount
    DESC
"""

import sqlite3

con = sqlite3.connect("../data/out/simulation.db")

cur = con.cursor()
cur.execute(sql)
rows = cur.fetchall()


def parse_ref(ref):
    if ref == "premaster":
        return 1
    elif ref == "master":
        return 2
    elif ref.startswith("gitlabCI/mergeRelease/"):
        return 3
    elif ref.startswith("release/"):
        return 4
    elif ref.startswith("gitlabCI/TPH-"):
        return 5
    else:
        # print("Unknown ref type: " + ref)
        return 0


def parse_status(status):
    if status == "failed":
        return 1
    elif status == "success":
        return 2
    elif status == "canceled":
        return 3
    else:
        print("Unknown status: " + status)
        return 0


print("status,ref_type,duration,is_relevant")
for (i, row) in enumerate(rows):
    status, duration, ref, raw_jobs, access_count = row
    jobs = raw_jobs.split(";")
    ref_num = parse_ref(ref)
    status_num = parse_status(status)
    is_relevant = 1 if access_count > 0 else 0
    print(str(status_num) + "," + str(ref_num) + "," + str(duration) + "," + str(is_relevant))
