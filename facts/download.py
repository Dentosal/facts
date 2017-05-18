import tempfile
import requests

def stream(url, existing_file=None, callback=(lambda a, b: None)):
    """Downloads and returns a temporary file containing the file."""
    assert callable(callback)

    if existing_file is None:
        tf = tempfile.TemporaryFile(mode="w+b")
    else:
        tf = existing_file

    r = requests.get(url, stream=True, timeout=30.0)
    file_size = r.headers.get("content-length", None)
    if file_size is not None:
        file_size = int(file_size)

    callback(0, file_size)

    try:
        downloaded_size = 0
        for chunk in r.iter_content(chunk_size=1024):
            if chunk: # filter out keep-alive new chunks
                tf.write(chunk)

                downloaded_size += len(chunk)
                callback(downloaded_size, file_size)
    except:
        tf.close()
        raise

    tf.seek(0)
    return tf
